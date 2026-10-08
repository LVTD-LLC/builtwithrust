//! End-to-end tests against the real router with a throwaway SQLite file.
//! Run with `cargo test`. No network access is needed: PostHog is disabled
//! and Stripe is only exercised through the webhook (signed locally).

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use builtwithrust::config::{Config, StripeConfig};
use builtwithrust::{SharedState, db};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

const TOKEN: &str = "test-admin-token";
const WEBHOOK_SECRET: &str = "whsec_test";

async fn app() -> (SharedState, Router, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("t.db");
    let cfg = Config {
        bind_addr: "127.0.0.1:0".into(),
        site_url: "http://test.local".into(),
        database_url: format!("sqlite://{}?mode=rwc", db_path.display()),
        admin_token: Some(TOKEN.into()),
        posthog: None,
        sentry_dsn: None,
        sentry_environment: "test".into(),
        stripe: Some(StripeConfig {
            secret_key: "sk_test_x".into(),
            webhook_secret: WEBHOOK_SECRET.into(),
            price_id: "price_x".into(),
        }),
        feature_days: 30,
        newsletter: None,
    };
    let (state, router) = builtwithrust::build(cfg).await.unwrap();
    (state, router, dir)
}

async fn send(router: &Router, req: Request<Body>) -> (StatusCode, axum::http::HeaderMap, String) {
    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let headers = resp.headers().clone();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    (status, headers, String::from_utf8_lossy(&body).into_owned())
}

fn get(path: &str) -> Request<Body> {
    Request::get(path).body(Body::empty()).unwrap()
}

fn admin_json(method: &str, path: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(path)
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn form(path: &str, body: &str) -> Request<Body> {
    Request::post(path)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn sample_project(slug: &str) -> Value {
    json!({
        "slug": slug, "name": "Sample Tool", "tagline": "A sample tagline.",
        "website_url": "https://sample.example", "repo_url": "https://github.com/x/y",
        "stars": 1234, "license": "MIT", "stack": ["axum", "tokio"],
        "category": "developer-tools", "verified": true
    })
}

#[tokio::test]
async fn public_pages_render() {
    let (_state, app, _dir) = app().await;
    for path in ["/", "/categories", "/submit", "/feature", "/sitemap.xml", "/robots.txt", "/healthz"] {
        let (status, _, _) = send(&app, get(path)).await;
        assert_eq!(status, StatusCode::OK, "{path}");
    }
    let (status, _, body) = send(&app, get("/projects/nope")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains("Not found"));

    let (_, headers, body) = send(&app, get("/healthz")).await;
    assert_eq!(body, "ok");
    assert_eq!(headers[header::CACHE_CONTROL], "no-store");
    assert_eq!(headers["x-deployment-revision"], option_env!("BUILD_REVISION").unwrap_or("development"));
}

#[tokio::test]
async fn admin_requires_token() {
    let (_state, app, _dir) = app().await;
    let (status, _, _) = send(&app, get("/api/admin/stats")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let bad =
        Request::get("/api/admin/stats").header(header::AUTHORIZATION, "Bearer wrong").body(Body::empty()).unwrap();
    let (status, _, _) = send(&app, bad).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _, body) = send(&app, admin_json("GET", "/api/admin/stats", json!({}))).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("\"projects\":0"));
}

#[tokio::test]
async fn upsert_project_appears_and_cache_purges() {
    let (state, app, _dir) = app().await;
    // Warm the cache with an empty home page.
    let (_, _, body) = send(&app, get("/")).await;
    assert!(!body.contains("Sample Tool"));
    assert!(!state.cache.is_empty());

    let (status, _, body) = send(&app, admin_json("POST", "/api/admin/projects", sample_project("sample-tool"))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let p: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(p["slug"], "sample-tool");
    assert_eq!(p["category_name"], "Developer Tools");

    let (_, _, body) = send(&app, get("/")).await;
    assert!(body.contains("Sample Tool"), "home should show the new project after purge");
    let (status, _, body) = send(&app, get("/projects/sample-tool")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("1.2k"));
    assert!(body.contains("tokio"));
    let (status, _, _) = send(&app, get("/categories/developer-tools")).await;
    assert_eq!(status, StatusCode::OK);

    // Update in place: same slug, new name.
    let mut updated = sample_project("sample-tool");
    updated["name"] = json!("Renamed Tool");
    let (status, _, _) = send(&app, admin_json("POST", "/api/admin/projects", updated)).await;
    assert_eq!(status, StatusCode::OK);
    let (_, _, body) = send(&app, get("/projects/sample-tool")).await;
    assert!(body.contains("Renamed Tool"));

    // Search is live and uncached.
    let (_, _, body) = send(&app, get("/?q=renamed")).await;
    assert!(body.contains("Renamed Tool"));
    let (_, _, body) = send(&app, get("/?q=zzzzzz")).await;
    assert!(body.contains("Nothing here yet"));

    // Delete.
    let (status, _, _) = send(&app, admin_json("DELETE", "/api/admin/projects/sample-tool", json!({}))).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _, _) = send(&app, get("/projects/sample-tool")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn invalid_project_is_rejected() {
    let (_state, app, _dir) = app().await;
    let bad = json!({ "name": "X", "tagline": "y", "website_url": "ftp://nope" });
    let (status, _, body) = send(&app, admin_json("POST", "/api/admin/projects", bad)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.contains("website_url"));
}

#[tokio::test]
async fn cached_pages_send_etag_and_compression() {
    let (_state, app, _dir) = app().await;
    let req = Request::get("/").header(header::ACCEPT_ENCODING, "gzip, br").body(Body::empty()).unwrap();
    let (status, headers, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers.get(header::CONTENT_ENCODING).unwrap(), "br");
    let etag = headers.get(header::ETAG).unwrap().clone();
    let req = Request::get("/").header(header::IF_NONE_MATCH, etag).body(Body::empty()).unwrap();
    let (status, _, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::NOT_MODIFIED);
}

#[tokio::test]
async fn submission_flow() {
    let (_state, app, _dir) = app().await;
    // Honeypot filled: accepted silently, nothing stored.
    let (status, _, _) = send(&app, form("/submit", "url=https%3A%2F%2Fspam.example&website=bot")).await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    // Bad URL.
    let (status, _, body) = send(&app, form("/submit", "url=not-a-url")).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("full URL"));
    // Good.
    let (status, headers, _) =
        send(&app, form("/submit", "url=https%3A%2F%2Fgood.example&email=a%40b.co&note=hi")).await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_eq!(headers.get(header::LOCATION).unwrap(), "/submit/thanks");

    let (status, _, body) = send(&app, admin_json("GET", "/api/admin/submissions", json!({}))).await;
    assert_eq!(status, StatusCode::OK);
    let subs: Vec<Value> = serde_json::from_str(&body).unwrap();
    assert_eq!(subs.len(), 1, "honeypot submission must not be stored");
    assert_eq!(subs[0]["url"], "https://good.example");
    assert_eq!(subs[0]["status"], "pending");
    let id = subs[0]["id"].as_i64().unwrap();

    // Approve and create the listing in one call.
    let review = json!({ "status": "approved", "project": sample_project("good") });
    let (status, _, body) =
        send(&app, admin_json("POST", &format!("/api/admin/submissions/{id}/review"), review)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let v: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["submission"]["status"], "approved");
    assert_eq!(v["submission"]["project_id"], v["project"]["id"]);
    let (status, _, _) = send(&app, get("/projects/good")).await;
    assert_eq!(status, StatusCode::OK);

    // Rate limit: 5 per day per IP.
    for _ in 0..4 {
        let (status, _, _) = send(&app, form("/submit", "url=https%3A%2F%2Fx.example")).await;
        assert_eq!(status, StatusCode::SEE_OTHER);
    }
    let (status, _, body) = send(&app, form("/submit", "url=https%3A%2F%2Fx.example")).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("try again tomorrow"));
}

#[tokio::test]
async fn stripe_webhook_features_project_once() {
    let (state, app, _dir) = app().await;
    let (_, _, body) = send(&app, admin_json("POST", "/api/admin/projects", sample_project("paid"))).await;
    let p: Value = serde_json::from_str(&body).unwrap();
    let project_id = p["id"].as_i64().unwrap();
    db::create_payment(&state.pool, "cs_test_123", project_id, Some("a@b.co")).await.unwrap();

    let event = json!({
        "id": "evt_1", "type": "checkout.session.completed",
        "data": { "object": {
            "id": "cs_test_123", "payment_status": "paid", "payment_intent": "pi_1",
            "amount_total": 4900, "currency": "usd", "metadata": { "project_slug": "paid" }
        }}
    });
    let payload = event.to_string();
    let stripe = state.stripe.as_ref().unwrap();
    let ts = time::OffsetDateTime::now_utc().unix_timestamp();

    // Wrong signature.
    let req = Request::post("/api/stripe/webhook")
        .header("stripe-signature", format!("t={ts},v1=deadbeef"))
        .body(Body::from(payload.clone()))
        .unwrap();
    let (status, _, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Right signature.
    let sig = stripe.sign_for_test(payload.as_bytes(), ts);
    let req = Request::post("/api/stripe/webhook")
        .header("stripe-signature", &sig)
        .body(Body::from(payload.clone()))
        .unwrap();
    let (status, _, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    let project = db::project_by_slug(&state.pool, "paid").await.unwrap().unwrap();
    assert!(project.is_featured());
    let first_until = project.featured_until.clone().unwrap();

    // Replay of the same event id is ignored (no double extension).
    let req = Request::post("/api/stripe/webhook")
        .header("stripe-signature", &sig)
        .body(Body::from(payload.clone()))
        .unwrap();
    let (status, _, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    let project = db::project_by_slug(&state.pool, "paid").await.unwrap().unwrap();
    assert_eq!(project.featured_until.unwrap(), first_until);

    // Featured projects appear once, in the normal listing.
    let (_, _, body) = send(&app, get("/")).await;
    assert!(!body.contains("Featured projects"));
    assert_eq!(body.matches("href=\"/projects/paid\"").count(), 1);
    assert!(body.contains("class=\"card card-featured\""));
    // Success page resolves the project from the session id.
    let (status, _, body) = send(&app, get("/feature/success?session_id=cs_test_123")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("Sample Tool"));
}

#[tokio::test]
async fn admin_can_feature_manually() {
    let (state, app, _dir) = app().await;
    send(&app, admin_json("POST", "/api/admin/projects", sample_project("manual"))).await;
    let (status, _, body) =
        send(&app, admin_json("POST", "/api/admin/projects/manual/feature", json!({ "days": 7 }))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let project = db::project_by_slug(&state.pool, "manual").await.unwrap().unwrap();
    assert!(project.is_featured());
    let mut regular = sample_project("regular");
    regular["stars"] = json!(999999);
    send(&app, admin_json("POST", "/api/admin/projects", regular)).await;
    for path in ["/", "/?q=Sample", "/categories/developer-tools"] {
        let (status, _, body) = send(&app, get(path)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.matches("href=\"/projects/manual\"").count(), 1);
        assert!(body.find("/projects/manual").unwrap() < body.find("/projects/regular").unwrap());
        assert_eq!(body.matches("class=\"card card-featured\"").count(), 1);
    }
    let (status, _, _) =
        send(&app, admin_json("POST", "/api/admin/projects/manual/feature", json!({ "days": 0 }))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn indexnow_ownership_is_public_plain_text() {
    let (_state, router, _dir) = app().await;
    let key = include_str!("../indexnow-key.txt").trim();
    let (status, headers, body) = send(&router, get(&format!("/{key}.txt"))).await;
    assert_eq!(status, StatusCode::OK);
    assert!(headers[header::CONTENT_TYPE].to_str().unwrap().starts_with("text/plain"));
    assert_eq!(body.trim(), key);
    let (status, _, _) = send(&router, get("/not-an-indexnow-key.txt")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn sitemap_preserves_full_modification_timestamp() {
    let (state, router, _dir) = app().await;
    let (status, _, _) =
        send(&router, admin_json("POST", "/api/admin/projects", sample_project("timestamp-check"))).await;
    assert_eq!(status, StatusCode::OK);
    let projects = db::published_slugs(&state.pool).await.unwrap();
    let (_, updated) = projects.iter().find(|(slug, _)| slug == "timestamp-check").unwrap();
    assert!(updated.contains('T'));
    let (status, _, body) = send(&router, get("/sitemap.xml")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(&format!("<lastmod>{updated}</lastmod>")));
}

// Distinct fixtures make combined filters, unknown metadata, and explicit sorting observable.
async fn browse_catalog(app: &Router) {
    for (slug, name, stars, category, stack, license, repo, verified, published) in [
        ("alpha", "Alpha", Some(9000), "tools", "tokio", "MIT", Some("https://github.com/x/a"), true, true),
        ("beta", "beta", Some(20), "tools", "tokio", "MIT", Some("https://github.com/x/b"), true, true),
        ("gamma", "Gamma", None, "tools", "axum", "Apache-2.0", None, false, true),
        ("delta", "Delta", Some(0), "apps", "tokio", "MIT", Some("https://github.com/x/d"), true, true),
        ("secret", "Secret", Some(1), "private", "private-stack", "private-license", None, true, false),
    ] {
        let payload = json!({ "slug": slug, "name": name, "tagline": "Rust catalog fixture", "website_url": "https://example.com", "stars": stars, "category": category, "stack": [stack], "license": license, "repo_url": repo, "verified": verified, "published": published });
        let (status, _, _) = send(app, admin_json("POST", "/api/admin/projects", payload)).await;
        assert_eq!(status, StatusCode::OK);
    }
    let (status, _, _) = send(app, admin_json("POST", "/api/admin/projects/gamma/feature", json!({"days":30}))).await;
    assert_eq!(status, StatusCode::OK);
}

fn card_slugs(body: &str) -> Vec<&str> {
    body.split("href=\"/projects/").skip(1).map(|s| s.split('"').next().unwrap()).collect()
}

#[tokio::test]
async fn directory_filters_combine_and_preserve_selected_values() {
    let (state, app, _dir) = app().await;
    browse_catalog(&app).await;
    let (_, _, default) = send(&app, get("/")).await;
    assert_eq!(card_slugs(&default), ["gamma", "alpha", "beta", "delta"]);
    assert!(!default.contains("private-stack"));
    assert!(!default.contains("private-license"));
    let cache_size = state.cache.len();
    let path = "/?q=rust&category=tools&stack=tokio&license=MIT&source=available&verified=1&stars=under-1000&sort=name";
    let (status, headers, body) = send(&app, get(path)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(card_slugs(&body), ["beta"]);
    assert!(body.contains("value=\"tokio\" selected"));
    assert!(body.contains("value=\"MIT\" selected"));
    assert!(body.contains("value=\"1\" checked"));
    assert!(body.contains("<strong>1</strong> of 4 projects"));
    assert_eq!(headers[header::CACHE_CONTROL], "no-store");
    assert_eq!(state.cache.len(), cache_size, "arbitrary combinations must not grow the cache");
    let (_, _, body) = send(&app, get("/?source=unlisted")).await;
    assert_eq!(card_slugs(&body), ["gamma"]);
    let (_, _, body) = send(&app, get("/?stars=1000")).await;
    assert_eq!(card_slugs(&body), ["alpha"]);
    let (_, _, body) = send(&app, get("/?stars=10000")).await;
    assert!(card_slugs(&body).is_empty());
    assert!(body.contains("reset all filters"));
    let (_, _, body) = send(&app, get("/?stack=to")).await;
    assert!(card_slugs(&body).is_empty(), "stack filtering uses exact values, not substrings");
    let (_, _, body) = send(&app, get("/?category=unknown")).await;
    assert!(card_slugs(&body).is_empty());
    assert!(body.contains("unknown (unavailable)"));
}

#[tokio::test]
async fn directory_sorts_override_featured_and_category_cache_cannot_leak() {
    let (state, app, _dir) = app().await;
    browse_catalog(&app).await;
    for (path, expected) in [
        ("/?sort=stars", vec!["alpha", "beta", "delta", "gamma"]),
        ("/?sort=name", vec!["alpha", "beta", "delta", "gamma"]),
        ("/?sort=gems", vec!["delta", "beta", "alpha"]),
        ("/categories/tools", vec!["gamma", "alpha", "beta"]),
        ("/categories/tools?q=rust&sort=gems&category=apps", vec!["beta", "alpha"]),
        ("/categories/tools?sort=stars", vec!["alpha", "beta", "gamma"]),
        ("/categories/tools", vec!["gamma", "alpha", "beta"]),
        ("/?sort=invalid&stars=garbage", vec!["gamma", "alpha", "beta", "delta"]),
    ] {
        let (status, _, body) = send(&app, get(path)).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        assert_eq!(card_slugs(&body), expected, "{path}");
    }
    let mut boundary = sample_project("boundary");
    boundary["stars"] = json!(10000);
    send(&app, admin_json("POST", "/api/admin/projects", boundary)).await;
    let (_, _, body) = send(&app, get("/?sort=gems")).await;
    assert_eq!(card_slugs(&body), ["delta", "beta", "alpha"]);
    send(&app, admin_json("DELETE", "/api/admin/projects/boundary", json!({}))).await;
    let mut projects = db::list_projects(&state.pool, None, None).await.unwrap();
    for p in &mut projects {
        p.created_at = if p.slug == "beta" { "2026-10-08T10:00:00Z" } else { "2026-01-01T00:00:00Z" }.into();
    }
    let browse = builtwithrust::directory::Browse { sort: "newest".into(), ..Default::default() };
    browse.apply(&mut projects);
    assert_eq!(projects.iter().map(|p| p.slug.as_str()).collect::<Vec<_>>(), ["beta", "alpha", "delta", "gamma"]);
    let (status, _, _) = send(&app, get("/categories/missing?sort=stars")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn directory_search_is_literal_and_escaped() {
    let (_state, app, _dir) = app().await;
    browse_catalog(&app).await;
    for path in ["/?q=%25", "/?q=%3Cscript%3E", "/?license=%22%3E%3Cscript%3E"] {
        let (status, _, body) = send(&app, get(path)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(card_slugs(&body).is_empty());
        assert!(!body.contains("value=\"\"><script>"));
        assert!(!body.contains("Results for “<script>”"));
    }
}
