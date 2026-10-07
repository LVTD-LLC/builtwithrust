use super::{html_now, ip_hash, page};
use crate::{SharedState, views::pages as v};
use axum::{
    extract::{Form, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use maud::html;
use serde::Deserialize;
use serde_json::{Value, json};
use std::time::Duration;

#[derive(Deserialize)]
pub struct Signup {
    #[serde(default)]
    email: String,
    #[serde(default)]
    website: String,
}

fn form_response(state: &SharedState, status: StatusCode, error: Option<&str>, email: &str) -> Response {
    let mut response = html_now(page(
        state,
        "Newsletter",
        "Weekly Rust news and projects.",
        "/newsletter",
        v::newsletter_form(error, email),
    ));
    *response.status_mut() = status;
    response
}

pub async fn form(State(state): State<SharedState>) -> Response {
    if state.cfg.newsletter.is_none() {
        return form_response(
            &state,
            StatusCode::SERVICE_UNAVAILABLE,
            Some("Signup is temporarily unavailable. Please try again later."),
            "",
        );
    }
    form_response(&state, StatusCode::OK, None, "")
}

fn success() -> Response {
    let mut response = Redirect::to("/newsletter/thanks").into_response();
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

pub async fn subscribe(State(state): State<SharedState>, headers: HeaderMap, Form(input): Form<Signup>) -> Response {
    // Browsers send Origin on same-origin POST forms. No per-visitor token in cached HTML.
    let expected = reqwest::Url::parse(&state.cfg.site_url).ok().map(|u| u.origin().ascii_serialization());
    let origin = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok());
    if origin.is_none() || origin != expected.as_deref() {
        return form_response(&state, StatusCode::FORBIDDEN, Some("Please submit the form from this website."), "");
    }
    let Some(cfg) = &state.cfg.newsletter else {
        return form_response(
            &state,
            StatusCode::SERVICE_UNAVAILABLE,
            Some("Signup is temporarily unavailable. Please try again later."),
            "",
        );
    };
    if !input.website.is_empty() {
        return success();
    }
    let email = input.email.trim();
    let valid = email.len() <= 254
        && !email.contains(char::is_whitespace)
        && !email.chars().any(char::is_control)
        && email.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty()
                && local.len() <= 64
                && !domain.contains('@')
                && domain.split('.').count() >= 2
                && domain.split('.').all(|label| !label.is_empty())
        });
    if !valid {
        return form_response(
            &state,
            StatusCode::UNPROCESSABLE_ENTITY,
            Some("Please enter a valid email address."),
            email,
        );
    }
    // CapRover overwrites X-Real-IP; don't trust visitor-supplied CF/X-Forwarded-For headers.
    // The application port must remain private behind that proxy.
    let ip = headers.get("x-real-ip").and_then(|v| v.to_str().ok()).unwrap_or("unknown");
    if !state.signup_limiter.allow(ip_hash(ip)) {
        let mut response = form_response(
            &state,
            StatusCode::TOO_MANY_REQUESTS,
            Some("Too many signup attempts. Please try again in an hour."),
            email,
        );
        response.headers_mut().insert(header::RETRY_AFTER, HeaderValue::from_static("3600"));
        return response;
    }
    let result = state
        .http
        .post(format!("{}/api/public/subscription", cfg.url))
        .timeout(Duration::from_secs(8))
        .json(&json!({"email": email, "name": "", "list_uuids": [&cfg.list_uuid]}))
        .send()
        .await;
    if let Ok(response) = result
        && response.status().is_success()
        && let Ok(body) = response.json::<Value>().await
        && body.get("data") == Some(&Value::Bool(true))
    {
        return success();
    }
    // Never log email, provider body or an exception containing request details.
    tracing::warn!("newsletter signup provider unavailable");
    form_response(
        &state,
        StatusCode::SERVICE_UNAVAILABLE,
        Some("We couldn't send your confirmation email. Please try again shortly."),
        email,
    )
}

pub async fn thanks(State(state): State<SharedState>) -> Response {
    html_now(page(
        &state,
        "Check your inbox",
        "Confirm your newsletter subscription.",
        "/newsletter/thanks",
        html! {
            section class="page narrow" {
                h1 { "Check your inbox" }
                p class="lede" { "If your address needs confirmation, you'll receive an email with a link to join. Already subscribed? You're all set." }
                a class="btn" href="/" { "Back to the directory" }
            }
        },
    ))
}
