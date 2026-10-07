use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::post,
};
use builtwithrust::config::{Config, NewsletterConfig};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

async fn app(upstream: Option<&str>) -> (Router, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let cfg = Config {
        bind_addr: "127.0.0.1:0".into(),
        site_url: "http://test.local".into(),
        database_url: format!("sqlite://{}?mode=rwc", dir.path().join("db").display()),
        admin_token: None,
        posthog: None,
        stripe: None,
        feature_days: 30,
        newsletter: upstream.map(|url| NewsletterConfig { url: url.into(), list_uuid: "fixed-list".into() }),
    };
    (builtwithrust::build(cfg).await.unwrap().1, dir)
}
fn signup(body: &str, origin: Option<&str>) -> Request<Body> {
    let mut req = Request::post("/newsletter").header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
    if let Some(origin) = origin {
        req = req.header(header::ORIGIN, origin);
    }
    req.body(Body::from(body.to_string())).unwrap()
}
async fn mock(status: StatusCode, result: Value) -> (String, Arc<Mutex<Vec<Value>>>, tokio::task::JoinHandle<()>) {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let capture = calls.clone();
    let server = Router::new().route(
        "/api/public/subscription",
        post(move |axum::Json(body): axum::Json<Value>| {
            let capture = capture.clone();
            let result = result.clone();
            async move {
                capture.lock().unwrap().push(body);
                (status, axum::Json(result))
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move { axum::serve(listener, server).await.unwrap() });
    (url, calls, task)
}
#[tokio::test]
async fn signup_requires_same_origin_valid_email_and_config() {
    let (router, _dir) = app(Some("http://127.0.0.1:1")).await;
    for origin in [None, Some("http://evil.example"), Some("null"), Some("http://test.local.evil.example")] {
        assert_eq!(
            router.clone().oneshot(signup("email=reader%40example.com", origin)).await.unwrap().status(),
            StatusCode::FORBIDDEN
        );
    }
    for email in ["bad", "a%40", "%40example.com", "a%40b%40c.com", "a%0Ab%40example.com"] {
        assert_eq!(
            router
                .clone()
                .oneshot(signup(&format!("email={email}"), Some("http://test.local")))
                .await
                .unwrap()
                .status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    let (disabled, _dir2) = app(None).await;
    assert_eq!(
        disabled.clone().oneshot(signup("email=a%40example.com", Some("http://test.local"))).await.unwrap().status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    let body = disabled
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap()
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    assert!(!String::from_utf8_lossy(&body).contains("action=\"/newsletter\""));
}
#[tokio::test]
async fn signup_uses_fixed_public_list_and_preserves_double_optin() {
    let (url, calls, task) = mock(StatusCode::OK, json!({"data": true})).await;
    let (router, _dir) = app(Some(&url)).await;
    for _ in 0..2 {
        let response = router
            .clone()
            .oneshot(signup("email=reader%40example.com&l=attacker&status=confirmed", Some("http://test.local")))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()[header::LOCATION], "/newsletter/thanks");
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    }
    assert_eq!(
        *calls.lock().unwrap(),
        vec![json!({"email":"reader@example.com", "name":"", "list_uuids":["fixed-list"]}); 2]
    );
    let response = router
        .clone()
        .oneshot(signup("email=bot%40example.com&website=spam", Some("http://test.local")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(calls.lock().unwrap().len(), 2);
    let home = router.oneshot(Request::get("/").body(Body::empty()).unwrap()).await.unwrap();
    assert!(home.headers()[header::CACHE_CONTROL].to_str().unwrap().contains("public"));
    let body = home.into_body().collect().await.unwrap().to_bytes();
    assert!(String::from_utf8_lossy(&body).contains("Weekly Rust news and projects"));
    task.abort();
}
#[tokio::test]
async fn provider_errors_are_retryable_and_never_echoed() {
    for (status, result) in [
        (StatusCode::BAD_GATEWAY, json!({"message":"private provider detail"})),
        (StatusCode::OK, json!({"data":false})),
    ] {
        let (url, _, task) = mock(status, result).await;
        let (router, _dir) = app(Some(&url)).await;
        let response = router.oneshot(signup("email=reader%40example.com", Some("http://test.local"))).await.unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = String::from_utf8_lossy(&body);
        assert!(body.contains("try again"));
        assert!(!body.contains("private provider detail"));
        task.abort();
    }
}
#[tokio::test]
async fn rate_limit_stops_delivery_attempts() {
    let (url, calls, task) = mock(StatusCode::OK, json!({"data":true})).await;
    let (router, _dir) = app(Some(&url)).await;
    for n in 0..10 {
        assert_eq!(
            router
                .clone()
                .oneshot(signup(&format!("email=reader{n}%40example.com"), Some("http://test.local")))
                .await
                .unwrap()
                .status(),
            StatusCode::SEE_OTHER
        );
    }
    let response = router.oneshot(signup("email=next%40example.com", Some("http://test.local"))).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(response.headers().contains_key(header::RETRY_AFTER));
    assert_eq!(calls.lock().unwrap().len(), 10);
    task.abort();
}
