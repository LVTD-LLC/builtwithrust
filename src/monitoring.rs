//! Bounded, non-blocking telemetry. Never attach requests, bodies, headers or search text.
use crate::config::Config;
use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};
use sentry::{Hub, SentryFutureExt, TransactionContext};
use std::{sync::Arc, time::Instant};

pub const RELEASE: &str = match option_env!("BUILD_REVISION") {
    Some(value) => value,
    None => "development",
};

pub fn scrub_event(mut event: sentry::protocol::Event<'static>) -> Option<sentry::protocol::Event<'static>> {
    event.request = None;
    event.user = None;
    event.extra.clear();
    event.breadcrumbs.values.clear();
    event.message = None;
    event.logentry = None;
    for exception in &mut event.exception.values {
        exception.value = Some("[redacted]".into());
        if let Some(stack) = &mut exception.stacktrace {
            for frame in &mut stack.frames {
                frame.vars.clear();
            }
        }
    }
    Some(event)
}

pub fn init(cfg: &Config) -> Option<sentry::ClientInitGuard> {
    let dsn = cfg.sentry_dsn.as_deref()?;
    Some(sentry::init(
        sentry::ClientOptions::new()
            .dsn(dsn)
            .environment(cfg.sentry_environment.clone())
            .release(RELEASE)
            .send_default_pii(false)
            .traces_sample_rate(0.2)
            .before_send(scrub_event),
    ))
}

pub async fn request(req: Request, next: Next) -> Response {
    // Disabled installs and static/health traffic retain the minimal hot path.
    if !Hub::current().client().is_some_and(|client| client.is_enabled()) {
        return next.run(req).await;
    }
    let route = req.extensions().get::<MatchedPath>().map(|p| p.as_str()).unwrap_or("/unmatched").to_owned();
    if matches!(route.as_str(), "/healthz" | "/assets/{*path}" | "/robots.txt" | "/sitemap.xml") {
        return next.run(req).await;
    }
    let method = match req.method().as_str() {
        "GET" => "GET",
        "POST" => "POST",
        "DELETE" => "DELETE",
        "HEAD" => "HEAD",
        _ => "OTHER",
    };
    let name = format!("{method} {route}");
    // Only the validated trace header, not baggage or any request metadata, is propagated.
    let parent = req.headers().get("sentry-trace").and_then(|v| v.to_str().ok());
    let context = TransactionContext::continue_from_headers(&name, "http.server", parent.map(|v| ("sentry-trace", v)));
    let hub = Arc::new(Hub::new_from_top(Hub::current()));
    async move {
        let transaction = sentry::start_transaction(context);
        sentry::configure_scope(|scope| {
            scope.set_span(Some(transaction.clone().into()));
            scope.set_tag("http.route", &route);
        });
        let start = Instant::now();
        let response = next.run(req).await;
        let status = response.status().as_u16();
        transaction.set_data("http.response.status_code", status.into());
        transaction.set_data("http.request.method", method.into());
        transaction.set_status(match status {
            401 => sentry::protocol::SpanStatus::Unauthenticated,
            403 => sentry::protocol::SpanStatus::PermissionDenied,
            404 => sentry::protocol::SpanStatus::NotFound,
            408 => sentry::protocol::SpanStatus::DeadlineExceeded,
            429 => sentry::protocol::SpanStatus::ResourceExhausted,
            400..=499 => sentry::protocol::SpanStatus::InvalidArgument,
            500..=599 => sentry::protocol::SpanStatus::InternalError,
            _ => sentry::protocol::SpanStatus::Ok,
        });
        sentry::metrics::counter("http.server.requests", 1)
            .attribute("http.route", route.clone())
            .attribute("http.response.status_code", status as i64)
            .capture();
        sentry::metrics::distribution("http.server.duration", start.elapsed().as_secs_f64() * 1000.0)
            .unit(sentry::protocol::Unit::Millisecond)
            .attribute("http.route", route.clone())
            .capture();
        // Successful requests only log when traced; failures always log. No free-text errors.
        if status >= 500 || transaction.iter_headers().any(|(_, value)| value.ends_with("-1")) {
            sentry::logger_info!(http.route = route, http.response.status_code = status, "HTTP request completed");
        }
        transaction.finish();
        response
    }
    .bind_hub(hub)
    .await
}

#[cfg(test)]
mod tests {
    #[test]
    fn error_payload_drops_private_data_and_retains_exception_type() {
        let event = serde_json::from_value(serde_json::json!({
            "message": "secret", "extra": {"token":"secret"},
            "request": {"url":"https://example.com/?secret", "data":"secret"},
            "user": {"email":"secret"},
            "exception": {"values":[{"type":"DatabaseError","value":"secret"}]},
            "breadcrumbs": {"values":[{"message":"secret"}]}
        }))
        .unwrap();
        let clean = super::scrub_event(event).unwrap();
        assert_eq!(clean.exception.values[0].ty, "DatabaseError");
        assert!(!serde_json::to_string(&clean).unwrap().contains("secret"));
    }
}

#[cfg(test)]
mod request_tests {
    use super::*;
    use axum::{Router, body::Body, middleware, routing::get};
    use tower::ServiceExt;
    #[tokio::test]
    async fn requests_keep_separate_traces_and_never_capture_query_or_headers() {
        let transport = sentry::test::TestTransport::new();
        let client = Arc::new(sentry::Client::from(
            sentry::ClientOptions::new()
                .dsn("https://public@sentry.invalid/1")
                .transport(transport.clone())
                .traces_sample_rate(1.0)
                .before_send(scrub_event),
        ));
        let hub = Arc::new(Hub::new(Some(client.clone()), Arc::new(Default::default())));
        let app = Router::new()
            .route(
                "/projects/{slug}",
                get(|| async {
                    tokio::task::yield_now().await;
                    crate::routes::AppError::Internal(anyhow::anyhow!("private error secret"))
                }),
            )
            .layer(middleware::from_fn(request));
        async {
            let a = app.clone().oneshot(
                Request::builder()
                    .uri("/projects/secret?q=secret")
                    .header("authorization", "Bearer secret")
                    .body(Body::empty())
                    .unwrap(),
            );
            let b = app.oneshot(Request::builder().uri("/projects/other?email=secret").body(Body::empty()).unwrap());
            let (a, b) = tokio::join!(a, b);
            assert_eq!(a.unwrap().status(), 500);
            assert_eq!(b.unwrap().status(), 500);
        }
        .bind_hub(hub)
        .await;
        client.flush(Some(std::time::Duration::from_secs(2)));
        let envelopes = transport.fetch_and_clear_envelopes();
        let events: Vec<_> = envelopes.iter().filter_map(|e| e.event()).collect();
        assert_eq!(events.len(), 2);
        assert_ne!(events[0].contexts.get("trace"), events[1].contexts.get("trace"));
        for envelope in envelopes {
            let mut bytes = Vec::new();
            envelope.to_writer(&mut bytes).unwrap();
            let text = String::from_utf8(bytes).unwrap();
            assert!(!text.contains("secret"), "private value in envelope");
        }
    }
}

/// Explicit operator-only CLI check, never a public HTTP route.
pub fn smoke() {
    let transaction = sentry::start_transaction(TransactionContext::new("monitoring.verification", "task"));
    sentry::configure_scope(|scope| {
        scope.set_span(Some(transaction.clone().into()));
        scope.set_tag("verification", "setup");
    });
    let error = anyhow::anyhow!("Controlled monitoring verification");
    let id = sentry::integrations::anyhow::capture_anyhow(&error);
    sentry::logger_info!("Monitoring verification");
    sentry::metrics::counter("monitoring.verification", 1).capture();
    transaction.finish();
    println!("Sentry verification event {id}");
}
