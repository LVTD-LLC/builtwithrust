//! HTTP routing. Handlers are grouped by audience:
//! - `pages`:  public HTML pages (cached)
//! - `api`:    public form posts, Stripe checkout + webhook, health, sitemap
//! - `admin`:  bearer-token JSON API used by agents to manage the catalog
//! - `assets`: embedded static files

pub mod admin;
pub mod api;
pub mod assets;
pub mod pages;

use crate::SharedState;
use crate::views;
use axum::extract::FromRequestParts;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header, request::Parts};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Router, extract::Request};
use maud::Markup;
use std::future::Future;
use std::net::SocketAddr;
use std::time::Duration;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/", get(pages::home))
        .route("/categories", get(pages::categories))
        .route("/categories/{slug}", get(pages::category))
        .route("/projects/{slug}", get(pages::project))
        .route("/submit", get(pages::submit).post(api::submit))
        .route("/submit/thanks", get(api::submit_thanks))
        .route("/feature", get(pages::feature))
        .route("/feature/success", get(pages::feature_success))
        .route("/feature/cancel", get(pages::feature_cancel))
        .route("/api/checkout", post(api::checkout))
        .route("/api/stripe/webhook", post(api::stripe_webhook))
        .route("/healthz", get(api::healthz))
        .route("/sitemap.xml", get(api::sitemap))
        .route("/robots.txt", get(api::robots))
        .route("/assets/{*path}", get(assets::serve))
        .nest("/api/admin", admin::router(state.clone()))
        .fallback(pages::not_found)
        .layer(middleware::from_fn(security_headers))
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(20)))
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .with_state(state)
}

async fn security_headers(req: Request, next: Next) -> Response {
    let mut resp = next.run(req).await;
    let h = resp.headers_mut();
    h.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    h.insert("x-frame-options", HeaderValue::from_static("DENY"));
    h.insert("referrer-policy", HeaderValue::from_static("strict-origin-when-cross-origin"));
    resp
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Any handler error. Logged, returned as a terse 500. Domain errors that
/// deserve a specific status use `AppError::Status`.
#[derive(Debug)]
pub enum AppError {
    Internal(anyhow::Error),
    Status(StatusCode, String),
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(e: E) -> Self {
        AppError::Internal(e.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Internal(e) => {
                tracing::error!(error = ?e, "request failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
            }
            AppError::Status(code, msg) => (code, msg).into_response(),
        }
    }
}

pub fn bad_request(msg: impl Into<String>) -> AppError {
    AppError::Status(StatusCode::BAD_REQUEST, msg.into())
}

// ---------------------------------------------------------------------------
// Cached page helper
// ---------------------------------------------------------------------------

const HTML: &str = "text/html; charset=utf-8";
/// Browsers revalidate every request (cheap: ETag -> 304); the CDN may hold
/// a copy for a minute. Writes purge the in-process cache; the CDN TTL is the
/// only staleness window.
const HTML_CACHE_CONTROL: &str = "public, max-age=0, s-maxage=60, stale-while-revalidate=300";

/// Serve a page from the render cache, rendering it on a miss.
pub async fn cached_html<F, Fut>(
    state: &SharedState,
    headers: &HeaderMap,
    key: String,
    render: F,
) -> Result<Response, AppError>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<Markup, AppError>>,
{
    if let Some(hit) = state.cache.get(&key) {
        return Ok(hit.respond(headers));
    }
    let markup = render().await?;
    let entry = state.cache.insert(key, bytes::Bytes::from(markup.into_string()), HTML, HTML_CACHE_CONTROL);
    Ok(entry.respond(headers))
}

/// Render a page without caching (forms with errors, 404s, search results).
pub fn html_now(markup: Markup) -> Response {
    let mut resp = markup.into_response();
    resp.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    resp
}

pub fn page<'a>(state: &SharedState, title: &'a str, description: &'a str, path: &'a str, body: Markup) -> Markup {
    views::layout(state, views::Page { title, description, path, body })
}

// ---------------------------------------------------------------------------
// Client IP extractor (behind a proxy or not)
// ---------------------------------------------------------------------------

pub struct ClientIp(pub String);

impl<S: Send + Sync> FromRequestParts<S> for ClientIp {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let from_header = ["cf-connecting-ip", "x-real-ip", "x-forwarded-for"]
            .iter()
            .filter_map(|h| parts.headers.get(*h))
            .filter_map(|v| v.to_str().ok())
            .map(|v| v.split(',').next().unwrap_or("").trim().to_string())
            .find(|v| !v.is_empty());
        let ip = from_header
            .or_else(|| parts.extensions.get::<axum::extract::ConnectInfo<SocketAddr>>().map(|c| c.0.ip().to_string()))
            .unwrap_or_else(|| "unknown".into());
        Ok(ClientIp(ip))
    }
}

/// Stable, non-reversible id for an anonymous visitor (used for rate
/// limiting and as the PostHog distinct_id for server-side events).
pub fn ip_hash(ip: &str) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(&Sha256::digest(format!("bwr:{ip}").as_bytes())[..12])
}
