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

#[derive(Default, Deserialize)]
pub struct SubmitForm {
    pub url: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub repo_url: String,
    #[serde(default)]
    pub twitter_url: String,
    pub feature: Option<String>,
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
        let body = v::submit_form(Some(msg), &form, state.cfg.payments_enabled());
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
    let repo = form.repo_url.trim();
    let twitter = form.twitter_url.trim();
    if !repo.is_empty() && !valid_social_url(repo, &["github.com", "www.github.com"], 2) {
        return Ok(render_err("Enter a GitHub repository URL, such as https://github.com/owner/project."));
    }
    if !twitter.is_empty() && !valid_social_url(twitter, &["twitter.com", "www.twitter.com", "x.com", "www.x.com"], 1) {
        return Ok(render_err("Enter the project's Twitter or X profile URL."));
    }
    if form.feature.is_some() && state.stripe.is_none() {
        return Ok(render_err("Checkout is unavailable. Uncheck the feature option to submit for free."));
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
        ((!repo.is_empty()).then_some(repo), (!twitter.is_empty()).then_some(twitter)),
        form.feature.is_some(),
    )
    .await?;
    state.posthog.capture(
        "submission_created",
        &hash,
        json!({ "submission_id": id, "url": url, "has_email": !email.is_empty() }),
    );
    if form.feature.is_some()
        && let Some(stripe) = &state.stripe
    {
        let thanks = format!("{}/submit/thanks", state.cfg.site_url);
        let checkout = stripe.create_checkout_session(&id.to_string(), "metadata[submission_id]", (!email.is_empty()).then_some(email), &thanks, &thanks)
            .await.map_err(|_| AppError::Status(StatusCode::BAD_GATEWAY, "Your submission is saved for review, but checkout could not start. You can feature the project once listed.".into()))?;
        db::register_submission_checkout(&state.pool, id, &checkout.id).await?;
        return Ok(Redirect::to(&checkout.url).into_response());
    }
    Ok(Redirect::to("/submit/thanks").into_response())
}

fn valid_social_url(value: &str, hosts: &[&str], segments: usize) -> bool {
    value.len() <= 2000
        && reqwest::Url::parse(value).is_ok_and(|url| {
            url.scheme() == "https"
                && url.username().is_empty()
                && url.password().is_none()
                && url.port().is_none()
                && url.host_str().is_some_and(|host| hosts.contains(&host))
                && url.path().trim_matches('/').split('/').filter(|s| !s.is_empty()).count() == segments
        })
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
    pub email: Option<String>,
}

pub async fn checkout(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    Form(form): Form<CheckoutForm>,
) -> Result<Response, AppError> {
    let Some(stripe) = &state.stripe else {
        return Err(AppError::Status(StatusCode::SERVICE_UNAVAILABLE, "Payments are unavailable.".into()));
    };
    let Some(project) = db::project_by_slug(&state.pool, form.project_slug.trim()).await?.filter(|p| p.published != 0)
    else {
        return Err(AppError::Status(StatusCode::NOT_FOUND, "Project not found.".into()));
    };
    let email = form.email.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let success_url = format!("{}/feature/success?session_id={{CHECKOUT_SESSION_ID}}", state.cfg.site_url);
    let cancel_url = format!("{}/projects/{}", state.cfg.site_url, project.slug);
    let session = stripe
        .create_checkout_session(&project.slug, "metadata[project_slug]", email, &success_url, &cancel_url)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "stripe checkout session failed");
            AppError::Status(StatusCode::BAD_GATEWAY, "Could not start checkout. Please try again.".into())
        })?;
    db::create_payment(&state.pool, &session.id, project.id, email).await?;
    state.posthog.capture(
        "checkout_started",
        &ip_hash(&ip),
        json!({ "project": project.slug, "session_id": session.id }),
    );
    Ok(Redirect::to(&session.url).into_response())
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
    if event.kind == "checkout.session.completed" || event.kind == "checkout.session.async_payment_succeeded" {
        let obj = &event.data.object;
        let paid = obj.get("payment_status").and_then(|v| v.as_str()) == Some("paid");
        let session_id = obj.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        if paid && !session_id.is_empty() {
            let payment_intent = obj.get("payment_intent").and_then(|v| v.as_str());
            let amount = obj.get("amount_total").and_then(|v| v.as_i64());
            let currency = obj.get("currency").and_then(|v| v.as_str());
            if amount != Some(29900) || currency != Some("usd") {
                return Ok(StatusCode::OK.into_response());
            }
            if let Some(pid) = db::mark_payment_paid(&state.pool, session_id, payment_intent, amount, currency).await? {
                state.cache.purge();
                state.posthog.capture(
                    "payment_completed",
                    "server",
                    json!({ "project_id": pid, "amount_cents": amount, "currency": currency, "session_id": session_id }),
                );
                tracing::info!(project_id = pid, session_id, "project featured via stripe");
            } else if let Some(intent) = db::rejected_payment(&state.pool, None, Some(session_id)).await? {
                stripe
                    .refund_rejected(&intent)
                    .await
                    .map_err(|_| AppError::Status(StatusCode::BAD_GATEWAY, "Refund pending; retry.".into()))?;
            }
        }
    }
    db::record_stripe_event(&state.pool, &event.id, &event.kind).await?;
    Ok(StatusCode::OK.into_response())
}

// ---------------------------------------------------------------------------
// Machine endpoints
// ---------------------------------------------------------------------------

pub async fn healthz(State(state): State<SharedState>) -> Result<Response, AppError> {
    db::project_count(&state.pool).await?;
    // Deployments must distinguish the new container from an older healthy one.
    let revision = option_env!("BUILD_REVISION").unwrap_or("development");
    Ok((StatusCode::OK, [("x-deployment-revision", revision), ("cache-control", "no-store")], "ok").into_response())
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
    xml.push_str(&format!("<url><loc>{base}/guides/dufs-vs-miniserve</loc><lastmod>2026-11-03</lastmod></url>"));
    // Catalog-driven hubs change independently of editorial releases; omit an untracked lastmod.
    for p in ["/", "/categories", "/submit"] {
        xml.push_str(&format!("<url><loc>{base}{p}</loc></url>"));
    }
    xml.push_str(&format!("<url><loc>{base}/guides</loc><lastmod>2026-11-24</lastmod></url>"));
    xml.push_str(&format!(
        "<url><loc>{base}/guides/ripgrep-hidden-ignored-files</loc><lastmod>2026-11-24</lastmod></url>"
    ));
    xml.push_str(&format!("<url><loc>{base}/guides/pueue-task-queue</loc><lastmod>2026-11-17</lastmod></url>"));
    xml.push_str(&format!(
        "<url><loc>{base}/guides/polars-csv-schema-inference</loc><lastmod>2026-11-01</lastmod></url>"
    ));
    xml.push_str(&format!("<url><loc>{base}/guides/rustic-backup-restore</loc><lastmod>2026-10-28</lastmod></url>"));
    xml.push_str(&format!(
        "<url><loc>{base}/guides/qsv-csv-validation</loc><lastmod>2026-10-27T12:06:00Z</lastmod></url>"
    ));
    xml.push_str(&format!(
        "<url><loc>{base}/guides/difftastic-vs-git-diff</loc><lastmod>2026-10-26T21:38:00Z</lastmod></url>"
    ));
    xml.push_str(&format!("<url><loc>{base}/guides/rust-python-tools</loc><lastmod>2026-10-10</lastmod></url>"));
    xml.push_str(&format!("<url><loc>{base}/guides/rust-terminal-tools</loc><lastmod>2026-10-11</lastmod></url>"));
    xml.push_str(&format!("<url><loc>{base}/guides/zed-vs-lapce</loc><lastmod>2026-10-13</lastmod></url>"));
    xml.push_str(&format!("<url><loc>{base}/guides/hurl-http-tests</loc><lastmod>2026-10-25</lastmod></url>"));
    for c in db::categories_with_counts(&state.pool).await? {
        xml.push_str(&format!("<url><loc>{base}/categories/{}</loc></url>", c.slug));
    }
    for (slug, updated) in db::published_slugs(&state.pool).await? {
        // These listings gained editorial links or metadata; preserve newer catalog edits.
        let updated = if slug == "octasine" {
            updated.max("2026-11-23T00:00:00Z".to_string())
        } else if slug == "spotify-player" {
            updated.max("2026-11-22T00:00:00Z".to_string())
        } else if slug == "bottom" {
            updated.max("2026-11-19T00:00:00Z".to_string())
        } else if matches!(slug.as_str(), "niri" | "rustdesk") {
            updated.max("2026-11-16T00:00:00Z".to_string())
        } else if slug == "rustpad" {
            updated.max("2026-11-20T00:00:00Z".to_string())
        } else if slug == "caligula" {
            updated.max("2026-11-07T00:00:00Z".to_string())
        } else if slug == "pueue" {
            updated.max("2026-11-17T00:00:00Z".to_string())
        } else if matches!(slug.as_str(), "dufs" | "miniserve" | "rqbit") {
            updated.max("2026-11-03T00:00:00Z".to_string())
        } else if slug == "polars" {
            updated.max("2026-11-01T00:00:00Z".to_string())
        } else if matches!(slug.as_str(), "uv" | "ruff") {
            updated.max("2026-10-10T00:00:00Z".to_string())
        } else if matches!(slug.as_str(), "alacritty" | "nushell" | "atuin" | "yazi") {
            updated.max("2026-10-11T00:00:00Z".to_string())
        } else if slug == "rustic" {
            updated.max("2026-10-28T00:00:00Z".to_string())
        } else if slug == "qsv" {
            updated.max("2026-10-27T12:06:00Z".to_string())
        } else if slug == "hurl" {
            updated.max("2026-10-25T00:00:00Z".to_string())
        } else if matches!(slug.as_str(), "difftastic" | "gitui") {
            updated.max("2026-10-26T21:38:00Z".to_string())
        } else if slug == "fish" {
            updated.max("2026-10-14T00:00:00Z".to_string())
        } else if matches!(slug.as_str(), "zed" | "lapce" | "rustpad") {
            updated.max("2026-10-29T00:00:00Z".to_string())
        } else {
            updated
        };
        xml.push_str(&format!("<url><loc>{base}/projects/{slug}</loc><lastmod>{}</lastmod></url>", updated));
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
