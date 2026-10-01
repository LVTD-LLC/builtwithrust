//! Public write endpoints and machine endpoints: submission form, Stripe
//! checkout + webhook, health, sitemap, robots.

use super::{AppError, ClientIp, bad_request, html_now, ip_hash, page};
use crate::SharedState;
use crate::db;
use crate::views::pages as v;
use axum::body::Bytes;
use axum::extract::{Form, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use serde_json::json;

const SUBMISSIONS_PER_DAY: i64 = 5;

#[derive(Deserialize)]
pub struct SubmitForm {
    pub url: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub note: String,
    /// Honeypot field, hidden from humans.
    #[serde(default)]
    pub website: String,
}

pub async fn submit(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    Form(form): Form<SubmitForm>,
) -> Result<Response, AppError> {
    let url = form.url.trim();
    let render_err = |msg: &str| {
        let body = v::submit_form(Some(msg), url);
        let mut resp = html_now(page(&state, "Submit a site", "Submit a site built with Rust.", "/submit", body));
        *resp.status_mut() = StatusCode::UNPROCESSABLE_ENTITY;
        resp
    };
    if !form.website.is_empty() {
        // Bot filled the honeypot. Pretend it worked.
        return Ok(Redirect::to("/submit/thanks").into_response());
    }
    if !(url.starts_with("http://") || url.starts_with("https://"))
        || url.len() > 2000
        || url.contains(char::is_whitespace)
    {
        return Ok(render_err("Please enter a full URL starting with http:// or https://."));
    }
    let email = form.email.trim();
    if !email.is_empty() && (!email.contains('@') || email.len() > 254) {
        return Ok(render_err("That email address does not look right."));
    }
    let note = form.note.trim().chars().take(2000).collect::<String>();
    let hash = ip_hash(&ip);
    if db::recent_submissions_from(&state.pool, &hash).await? >= SUBMISSIONS_PER_DAY {
        return Ok(render_err("You have submitted a lot today. Please try again tomorrow."));
    }
    let id = db::create_submission(
        &state.pool,
        url,
        (!email.is_empty()).then_some(email),
        (!note.is_empty()).then_some(note.as_str()),
        Some(&hash),
    )
    .await?;
    state.posthog.capture(
        "submission_created",
        &hash,
        json!({ "submission_id": id, "url": url, "has_email": !email.is_empty() }),
    );
    Ok(Redirect::to("/submit/thanks").into_response())
}

pub async fn submit_thanks(State(state): State<SharedState>) -> Response {
    html_now(page(&state, "Thanks", "Submission received.", "/submit/thanks", v::submit_thanks()))
}

// ---------------------------------------------------------------------------
// Stripe
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CheckoutForm {
    pub project_slug: String,
    pub email: String,
}

pub async fn checkout(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    Form(form): Form<CheckoutForm>,
) -> Result<Response, AppError> {
    let Some(stripe) = &state.stripe else {
        return feature_error(&state, StatusCode::SERVICE_UNAVAILABLE, "Payments are not enabled yet.").await;
    };
    let Some(project) = db::project_by_slug(&state.pool, form.project_slug.trim()).await? else {
        return feature_error(&state, StatusCode::UNPROCESSABLE_ENTITY, "That project is not listed. Submit it first.")
            .await;
    };
    let email = form.email.trim();
    if !email.contains('@') {
        return feature_error(&state, StatusCode::UNPROCESSABLE_ENTITY, "Please enter a valid email for the receipt.")
            .await;
    }
    let success_url = format!("{}/feature/success?session_id={{CHECKOUT_SESSION_ID}}", state.cfg.site_url);
    let cancel_url = format!("{}/feature/cancel", state.cfg.site_url);
    let session =
        stripe.create_checkout_session(&project.slug, Some(email), &success_url, &cancel_url).await.map_err(|e| {
            tracing::error!(error = %e, "stripe checkout session failed");
            AppError::Status(StatusCode::BAD_GATEWAY, "Could not start checkout. Please try again.".into())
        })?;
    db::create_payment(&state.pool, &session.id, project.id, Some(email)).await?;
    state.posthog.capture(
        "checkout_started",
        &ip_hash(&ip),
        json!({ "project": project.slug, "session_id": session.id }),
    );
    Ok(Redirect::to(&session.url).into_response())
}

async fn feature_error(state: &SharedState, status: StatusCode, msg: &str) -> Result<Response, AppError> {
    let projects = db::list_projects(&state.pool, None, None).await?;
    let body = v::feature(&projects, state.cfg.payments_enabled(), state.cfg.feature_days, Some(msg));
    let mut resp = html_now(page(state, "Feature your project", "Feature your project.", "/feature", body));
    *resp.status_mut() = status;
    Ok(resp)
}

pub async fn stripe_webhook(
    State(state): State<SharedState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    let Some(stripe) = &state.stripe else {
        return Ok(StatusCode::SERVICE_UNAVAILABLE.into_response());
    };
    let sig = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| bad_request("missing stripe-signature"))?;
    let event = stripe.parse_webhook(&body, sig).map_err(|e| {
        tracing::warn!(error = %e, "rejected stripe webhook");
        bad_request(e.to_string())
    })?;
    if !db::record_stripe_event(&state.pool, &event.id, &event.kind).await? {
        // Already processed: Stripe retries, we stay idempotent.
        return Ok(StatusCode::OK.into_response());
    }
    if event.kind == "checkout.session.completed" || event.kind == "checkout.session.async_payment_succeeded" {
        let obj = &event.data.object;
        let paid = obj.get("payment_status").and_then(|v| v.as_str()) == Some("paid");
        let session_id = obj.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        if paid && !session_id.is_empty() {
            let payment_intent = obj.get("payment_intent").and_then(|v| v.as_str());
            let amount = obj.get("amount_total").and_then(|v| v.as_i64());
            let currency = obj.get("currency").and_then(|v| v.as_str());
            let mut project_id =
                db::mark_payment_paid(&state.pool, session_id, payment_intent, amount, currency).await?;
            if project_id.is_none() {
                // Session was not created by us (or db was reset): fall back to metadata.
                if let Some(slug) = obj.pointer("/metadata/project_slug").and_then(|v| v.as_str())
                    && let Some(p) = db::project_by_slug(&state.pool, slug).await?
                {
                    project_id = Some(p.id);
                }
            }
            if let Some(pid) = project_id {
                db::extend_featured(&state.pool, pid, state.cfg.feature_days).await?;
                state.cache.purge();
                state.posthog.capture(
                    "payment_completed",
                    "server",
                    json!({ "project_id": pid, "amount_cents": amount, "currency": currency, "session_id": session_id }),
                );
                tracing::info!(project_id = pid, session_id, "project featured via stripe");
            } else {
                tracing::warn!(session_id, "paid session with no matching project");
            }
        }
    }
    Ok(StatusCode::OK.into_response())
}

// ---------------------------------------------------------------------------
// Machine endpoints
// ---------------------------------------------------------------------------

pub async fn healthz(State(state): State<SharedState>) -> Result<Response, AppError> {
    db::project_count(&state.pool).await?;
    Ok((StatusCode::OK, "ok").into_response())
}

pub async fn sitemap(State(state): State<SharedState>, headers: HeaderMap) -> Result<Response, AppError> {
    if let Some(hit) = state.cache.get("/sitemap.xml") {
        return Ok(hit.respond(&headers));
    }
    let base = &state.cfg.site_url;
    let mut xml = String::with_capacity(8192);
    xml.push_str(
        r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#,
    );
    for p in ["/", "/categories", "/submit", "/feature"] {
        xml.push_str(&format!("<url><loc>{base}{p}</loc></url>"));
    }
    for c in db::categories_with_counts(&state.pool).await? {
        xml.push_str(&format!("<url><loc>{base}/categories/{}</loc></url>", c.slug));
    }
    for (slug, updated) in db::published_slugs(&state.pool).await? {
        xml.push_str(&format!(
            "<url><loc>{base}/projects/{slug}</loc><lastmod>{}</lastmod></url>",
            &updated[..10.min(updated.len())]
        ));
    }
    xml.push_str("</urlset>");
    let entry = state.cache.insert("/sitemap.xml".into(), Bytes::from(xml), "application/xml", "public, max-age=3600");
    Ok(entry.respond(&headers))
}

pub async fn robots(State(state): State<SharedState>) -> Response {
    let body = format!("User-agent: *\nAllow: /\nDisallow: /api/\nSitemap: {}/sitemap.xml\n", state.cfg.site_url);
    let mut resp = body.into_response();
    resp.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/plain; charset=utf-8"));
    resp
}
