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
async fn fish_listing_distinguishes_shell_and_language_servers_without_leaking_to_other_projects() {
    let (_state, app, _dir) = app().await;
    let seed: Vec<Value> = serde_json::from_str(include_str!("../seed/projects.json")).unwrap();
    let fish = seed.into_iter().find(|p| p["slug"] == "fish").unwrap();
    let (status, _, _) = send(&app, admin_json("POST", "/api/admin/projects", fish)).await;
    assert_eq!(status, StatusCode::OK);
    let (status, headers, body) = send(&app, get("/projects/fish")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("shell implementation was ported from C++ to Rust"));
    assert!(!body.contains("entire codebase was ported"));
    assert!(body.contains("Fish, fish-lsp or rust-analyzer?"));
    assert!(body.contains("TypeScript language server"));
    assert!(body.contains("href=\"/guides/rust-terminal-tools\""));
    assert!(body.contains("href=\"http://test.local/projects/fish\""));
    assert_eq!(body.matches("<h1").count(), 1);
    let conditional = Request::get("/projects/fish")
        .header(header::IF_NONE_MATCH, &headers[header::ETAG])
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(&app, conditional).await.0, StatusCode::NOT_MODIFIED);
    send(&app, admin_json("POST", "/api/admin/projects", sample_project("unrelated"))).await;
    let (_, _, body) = send(&app, get("/projects/unrelated")).await;
    assert!(!body.contains("fish-tools-title"));
    let (_, _, sitemap) = send(&app, get("/sitemap.xml")).await;
    let modified = sitemap.split("/projects/fish</loc><lastmod>").nth(1).unwrap().split('<').next().unwrap();
    assert!(modified >= "2026-10-14T00:00:00Z");
}

#[tokio::test]
async fn database_catalog_sources_are_scoped_to_their_listings() {
    let (_state, app, _dir) = app().await;
    for (slug, expected) in [
        ("meilisearch", "Community and Enterprise code"),
        ("surrealdb", "FoundationDB support ended in 3.0.0"),
        ("other", ""),
    ] {
        send(&app, admin_json("POST", "/api/admin/projects", sample_project(slug))).await;
        let path = format!("/projects/{slug}");
        let (status, headers, body) = send(&app, get(&path)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.contains("id=\"source-scope-title\""), !expected.is_empty());
        assert!(body.contains(expected));
        if !expected.is_empty() {
            assert!(body.contains("href=\"/categories/databases-search#database-workflows\""));
        }
        let conditional =
            Request::get(&path).header(header::IF_NONE_MATCH, &headers[header::ETAG]).body(Body::empty()).unwrap();
        assert_eq!(send(&app, conditional).await.0, StatusCode::NOT_MODIFIED);
    }
}

#[tokio::test]
async fn warp_listing_scopes_licenses_and_distinguishes_framework_with_cached_rendering() {
    let (_state, app, _dir) = app().await;
    let seed: Vec<Value> = serde_json::from_str(include_str!("../seed/projects.json")).unwrap();
    let warp = seed.into_iter().find(|p| p["slug"] == "warp").unwrap();
    assert_eq!(send(&app, admin_json("POST", "/api/admin/projects", warp)).await.0, StatusCode::OK);
    let (status, headers, body) = send(&app, get("/projects/warp")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("AGPL-3.0 (client); MIT (UI crates)"));
    assert!(!body.contains("Proprietary"));
    assert!(body.contains("hosted-service terms"));
    assert!(body.contains("Warp terminal or the warp web framework?"));
    assert!(body.contains("href=\"https://docs.rs/warp/latest/warp/\""));
    assert!(body.contains("href=\"https://github.com/warpdotdev/warp#licensing\""));
    assert!(body.contains("href=\"/guides/rust-terminal-tools\""));
    assert!(body.contains("href=\"http://test.local/projects/warp\""));
    assert_eq!(body.matches("<h1").count(), 1);
    let conditional = Request::get("/projects/warp")
        .header(header::IF_NONE_MATCH, &headers[header::ETAG])
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(&app, conditional).await.0, StatusCode::NOT_MODIFIED);
    send(&app, admin_json("POST", "/api/admin/projects", sample_project("unrelated"))).await;
    let (_, _, body) = send(&app, get("/projects/unrelated")).await;
    assert!(!body.contains("warp-tools-title"));
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
async fn category_discovery_has_headings_unique_metadata_and_crawlable_links() {
    let (_state, app, _dir) = app().await;
    let mut project = sample_project("seo-tool");
    project["category_name"] = json!("Developer Tools");
    let (status, _, _) = send(&app, admin_json("POST", "/api/admin/projects", project)).await;
    assert_eq!(status, StatusCode::OK);
    for path in ["/categories/developer-tools", "/categories/developer-tools?sort=name"] {
        let (status, _, body) = send(&app, get(path)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.matches("<h1>").count(), 1);
        assert!(body.contains("<h1>Developer Tools built with Rust</h1>"));
        assert!(body.contains("content=\"Explore Developer Tools built with Rust."));
        assert!(body.contains("rel=\"canonical\" href=\"http://test.local/categories/developer-tools\""));
    }
    for _ in 0..2 {
        let (_, _, home) = send(&app, get("/")).await;
        assert_eq!(home.matches("<h1>").count(), 1);
        assert!(home.contains("aria-label=\"Browse projects by category\""));
        assert!(home.contains("href=\"/categories/developer-tools\""));
    }
    let (_, _, search) = send(&app, get("/?q=Sample")).await;
    assert_eq!(search.matches("<h1>").count(), 1);
    let (status, _, _) = send(&app, get("/categories/missing")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
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
    assert!(body.contains("Showing <strong>1–1</strong> of <strong>1</strong> projects"));
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

// A canonical-only chooser must not contaminate filtered HTML or its cached response.
#[tokio::test]
async fn category_workflow_discovery_is_cached_and_isolated_from_filters() {
    let (_state, app, _dir) = app().await;
    for (category, name, slugs, marker, date, distinctive, targets) in [
        (
            "productivity",
            "Productivity",
            ["typst", "inlyne"],
            "document-workflows",
            "2026-10-22",
            "Inlyne is not a full browser",
            vec!["/projects/typst", "/projects/inlyne"],
        ),
        (
            "productivity",
            "Productivity",
            ["presenterm", "other-productivity"],
            "presentation-workflows",
            "2026-10-23",
            "Snippet validation can also run code",
            vec!["/projects/presenterm"],
        ),
        (
            "developer-tools",
            "Developer Tools",
            ["biome", "rspack"],
            "javascript-workflows",
            "2026-10-21",
            "Keep TypeScript type checking explicit",
            vec!["/projects/biome", "/projects/rspack"],
        ),
        (
            "data-and-ai",
            "Data & AI",
            ["polars", "qsv"],
            "data-workflows",
            "2026-10-27T12:06:00Z",
            "qsv’s optional SQL commands use Polars",
            vec!["/projects/qsv", "/projects/polars", "/guides/rust-python-tools", "/guides/qsv-csv-validation"],
        ),
        (
            "infrastructure",
            "Infrastructure",
            ["gping", "trippy"],
            "network-workflows",
            "2026-10-18",
            "not interchangeable tests of application health",
            vec!["/projects/gping", "/projects/trippy"],
        ),
        (
            "infrastructure",
            "Infrastructure",
            ["dufs", "miniserve"],
            "file-sharing-workflows",
            "2026-10-19",
            "miniserve’s WebDAV support is read-only",
            vec!["/projects/dufs", "/projects/miniserve"],
        ),
        (
            "databases-search",
            "Databases & Search",
            ["meilisearch", "surrealdb"],
            "database-workflows",
            "2026-10-17",
            "Both support full-text search",
            vec!["/projects/meilisearch", "/projects/surrealdb"],
        ),
    ] {
        for slug in slugs {
            let mut project = sample_project(slug);
            project["category"] = json!(category);
            project["category_name"] = json!(name);
            assert_eq!(send(&app, admin_json("POST", "/api/admin/projects", project)).await.0, StatusCode::OK);
        }
        let path = format!("/categories/{category}");
        let (status, headers, body) = send(&app, get(&path)).await;
        assert_eq!(status, StatusCode::OK);
        if category == "infrastructure" {
            assert!(!body.contains("href=\"/projects/sniffnet\""));
            assert!(!body.contains("href=\"/projects/pingora\""));
        }
        let marker = format!("id=\"{marker}\"");
        assert!(body.contains(&marker));
        assert!(body.contains(distinctive));
        assert_eq!(body.matches("<h1>").count(), 1);
        assert!(body.contains(&format!("rel=\"canonical\" href=\"http://test.local{path}\"")));
        for target in targets {
            assert!(body.contains(&format!("href=\"{target}\"")));
            assert_eq!(send(&app, get(target)).await.0, StatusCode::OK);
        }
        let schemas: Vec<Value> = body
            .split("<script type=\"application/ld+json\">")
            .skip(1)
            .map(|part| serde_json::from_str(part.split("</script>").next().unwrap()).unwrap())
            .collect();
        if marker.contains("file-sharing")
            || marker.contains("javascript")
            || marker.contains("document")
            || marker.contains("presentation")
        {
            let schema = schemas
                .iter()
                .find(|s| {
                    s["@id"]
                        == format!(
                            "http://test.local{path}#{}",
                            marker.trim_start_matches("id=\"").trim_end_matches('"')
                        )
                })
                .unwrap();
            assert_eq!(schema["dateModified"], date);
            assert_eq!(schema["isPartOf"]["@id"], format!("http://test.local{path}"));
        } else {
            let schema = schemas.iter().find(|s| s["@type"] == "CollectionPage").unwrap();
            assert_eq!(schema["hasPart"]["dateModified"], date);
        }
        let conditional =
            Request::get(&path).header(header::IF_NONE_MATCH, &headers[header::ETAG]).body(Body::empty()).unwrap();
        assert_eq!(send(&app, conditional).await.0, StatusCode::NOT_MODIFIED);
        for filtered in [
            "/".to_owned(),
            format!("/?category={category}"),
            format!("{path}?q=missing"),
            format!("{path}?sort=name"),
            format!("{path}?license=MIT"),
        ] {
            let (status, _, body) = send(&app, get(&filtered)).await;
            assert_eq!(status, StatusCode::OK);
            assert!(!body.contains(&marker), "{filtered}");
        }
        assert!(send(&app, get(&path)).await.2.contains(&marker));
        if marker.contains("presentation") {
            let mut project = sample_project("presenterm");
            project["category"] = json!(category);
            project["published"] = json!(false);
            assert_eq!(send(&app, admin_json("POST", "/api/admin/projects", project.clone())).await.0, StatusCode::OK);
            assert!(
                !send(&app, get(&path)).await.2.contains(&marker),
                "unpublishing purges presentation recommendations"
            );
            project["published"] = json!(true);
            project["category"] = json!("other-category");
            assert_eq!(send(&app, admin_json("POST", "/api/admin/projects", project)).await.0, StatusCode::OK);
            assert!(
                !send(&app, get(&path)).await.2.contains(&marker),
                "a moved project must not remain recommended here"
            );
        }
        if marker.contains("document") {
            for slug in ["typst", "inlyne"] {
                let mut project = sample_project(slug);
                project["category"] = json!(category);
                project["published"] = json!(false);
                assert_eq!(send(&app, admin_json("POST", "/api/admin/projects", project)).await.0, StatusCode::OK);
                let (_, _, body) = send(&app, get(&path)).await;
                assert!(!body.contains(&format!("href=\"/projects/{slug}\"")), "unpublish purges recommendations");
            }
            assert!(
                !send(&app, get(&path)).await.2.contains(&marker),
                "hide the chooser when neither project is published"
            );
        }
        if marker.contains("javascript") {
            assert!(!body.contains("href=\"/projects/turbopack\""), "never recommend an absent listing");
            let mut project = sample_project("biome");
            project["category"] = json!(category);
            project["published"] = json!(false);
            assert_eq!(send(&app, admin_json("POST", "/api/admin/projects", project)).await.0, StatusCode::OK);
            let (_, _, body) = send(&app, get(&path)).await;
            assert!(!body.contains("href=\"/projects/biome\""), "unpublishing purges cached recommendations");
            assert!(body.contains("href=\"/projects/rspack\""));
        }
        if marker.contains("file-sharing") {
            let mut project = sample_project("dufs");
            project["category"] = json!(category);
            project["published"] = json!(false);
            assert_eq!(send(&app, admin_json("POST", "/api/admin/projects", project)).await.0, StatusCode::OK);
            let (_, _, body) = send(&app, get(&path)).await;
            assert!(!body.contains("href=\"/projects/dufs\""));
            assert!(body.contains("href=\"/projects/miniserve\""));
        }
        if category == "infrastructure" && marker.contains("network") {
            // A catalog unpublish must purge both the card grid and cached task recommendations.
            let mut project = sample_project("gping");
            project["category"] = json!(category);
            project["published"] = json!(false);
            assert_eq!(send(&app, admin_json("POST", "/api/admin/projects", project)).await.0, StatusCode::OK);
            let (_, _, body) = send(&app, get(&path)).await;
            assert!(!body.contains("href=\"/projects/gping\""));
            assert!(body.contains("href=\"/projects/trippy\""));
        }
    }
}

#[tokio::test]
async fn developer_guide_is_cached_and_does_not_leak_into_filtered_views() {
    let (_state, app, _dir) = app().await;
    let (status, _, _) = send(&app, admin_json("POST", "/api/admin/projects", sample_project("guide-test"))).await;
    assert_eq!(status, StatusCode::OK);
    let path = "/categories/developer-tools";
    let (_, _, body) = send(&app, get(path)).await;
    assert!(body.contains("id=\"choose-by-task\""));
    assert!(!body.contains("id=\"git-workflows\""));
    // Catalog changes must invalidate the cached hub and expose only eligible listings.
    let (status, _, _) = send(&app, admin_json("POST", "/api/admin/projects", sample_project("gitui"))).await;
    assert_eq!(status, StatusCode::OK);
    let (_, headers, body) = send(&app, get(path)).await;
    assert!(body.contains("id=\"git-workflows\""));
    assert!(body.contains("href=\"/projects/gitui\""));
    assert!(body.contains("uv manages Python projects"));
    assert_eq!(body.matches("<h1>").count(), 1);
    let schema: Value = body
        .split("<script type=\"application/ld+json\">")
        .skip(1)
        .map(|part| serde_json::from_str::<Value>(part.split("</script>").next().unwrap()).unwrap())
        .find(|value| value["@type"] == "CollectionPage")
        .unwrap();
    assert_eq!(schema["@type"], "CollectionPage");
    assert_eq!(schema["url"], "http://test.local/categories/developer-tools");
    assert_eq!(schema["hasPart"]["dateModified"], "2026-10-12");
    assert!(body.contains("aria-label=\"Developer workflow guides\""));
    for guide in ["/guides/rust-python-tools", "/guides/rust-terminal-tools"] {
        assert!(body.contains(&format!("href=\"{guide}\"")));
        assert_eq!(send(&app, get(guide)).await.0, StatusCode::OK);
    }
    let etag = headers.get(header::ETAG).unwrap();
    let conditional = Request::get(path).header(header::IF_NONE_MATCH, etag).body(Body::empty()).unwrap();
    assert_eq!(send(&app, conditional).await.0, StatusCode::NOT_MODIFIED);
    for filtered in [
        "/",
        "/?category=developer-tools",
        "/categories/developer-tools?q=missing",
        "/categories/developer-tools?sort=name",
    ] {
        let (status, _, body) = send(&app, get(filtered)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(!body.contains("id=\"choose-by-task\""), "{filtered}");
        assert!(!body.contains("id=\"git-workflows\""), "{filtered}");
        assert!(!body.contains("aria-label=\"Developer workflow guides\""), "{filtered}");
    }
    let (status, _, body) = send(&app, get(path)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("id=\"choose-by-task\""));
    let unpublished = serde_json::json!({"published": false});
    let mut project = sample_project("gitui");
    project.as_object_mut().unwrap().extend(unpublished.as_object().unwrap().clone());
    assert_eq!(send(&app, admin_json("POST", "/api/admin/projects", project)).await.0, StatusCode::OK);
    assert!(!send(&app, get(path)).await.2.contains("id=\"git-workflows\""));
}

#[tokio::test]
async fn python_guide_is_discoverable_canonical_and_cached() {
    let (state, app, _dir) = app().await;
    for slug in ["uv", "ruff", "polars", "unrelated-tool"] {
        let (status, _, _) = send(&app, admin_json("POST", "/api/admin/projects", sample_project(slug))).await;
        assert_eq!(status, StatusCode::OK);
    }
    let path = "/guides/rust-python-tools";
    let (status, headers, body) = send(&app, get(path)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.matches("<h1>").count(), 1);
    assert!(body.contains("<h1>Rust-powered Python tools: uv, Ruff and Polars</h1>"));
    // SVG shapes must close explicitly or later labels become invisible children.
    assert_eq!(body.matches("</rect>").count(), 3);
    assert!(body.contains("rel=\"canonical\" href=\"http://test.local/guides/rust-python-tools\""));
    assert!(state.cache.get(path).is_some());
    let etag = headers.get(header::ETAG).unwrap();
    let conditional = Request::get(path).header(header::IF_NONE_MATCH, etag).body(Body::empty()).unwrap();
    let (status, _, body) = send(&app, conditional).await;
    assert_eq!(status, StatusCode::NOT_MODIFIED);
    assert!(body.is_empty());
    let (_, _, sitemap) = send(&app, get("/sitemap.xml")).await;
    assert!(
        sitemap
            .contains("<url><loc>http://test.local/guides/rust-python-tools</loc><lastmod>2026-10-10</lastmod></url>")
    );
    for inbound in ["/guides", "/categories/developer-tools", "/projects/uv", "/projects/ruff", "/projects/polars"] {
        let (status, _, body) = send(&app, get(inbound)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("href=\"/guides/rust-python-tools\""), "{inbound}");
    }
    for unrelated in ["/?q=uv", "/categories/developer-tools?sort=name", "/projects/unrelated-tool"] {
        let (status, _, body) = send(&app, get(unrelated)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(!body.contains("href=\"/guides/rust-python-tools\""), "{unrelated}");
    }
}

#[tokio::test]
async fn terminal_guide_is_discoverable_canonical_and_cached() {
    let (state, app, _dir) = app().await;
    for slug in ["alacritty", "nushell", "atuin", "yazi", "unrelated-tool"] {
        let (status, _, _) = send(&app, admin_json("POST", "/api/admin/projects", sample_project(slug))).await;
        assert_eq!(status, StatusCode::OK);
    }
    let path = "/guides/rust-terminal-tools";
    let (status, headers, body) = send(&app, get(path)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.matches("<h1>").count(), 1);
    assert!(body.contains("<article class=\"terminal-guide python-guide page\">"));
    assert!(body.contains("rel=\"canonical\" href=\"http://test.local/guides/rust-terminal-tools\""));
    assert!(body.contains("property=\"og:type\" content=\"article\""));
    assert!(state.cache.get(path).is_some());
    let etag = headers.get(header::ETAG).unwrap();
    let conditional = Request::get(path).header(header::IF_NONE_MATCH, etag).body(Body::empty()).unwrap();
    let (status, _, body) = send(&app, conditional).await;
    assert_eq!(status, StatusCode::NOT_MODIFIED);
    assert!(body.is_empty());
    let (_, _, sitemap) = send(&app, get("/sitemap.xml")).await;
    assert!(
        sitemap.contains(
            "<url><loc>http://test.local/guides/rust-terminal-tools</loc><lastmod>2026-10-11</lastmod></url>"
        )
    );
    for inbound in [
        "/guides",
        "/categories/developer-tools",
        "/projects/alacritty",
        "/projects/nushell",
        "/projects/atuin",
        "/projects/yazi",
    ] {
        let (status, _, body) = send(&app, get(inbound)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("href=\"/guides/rust-terminal-tools\""), "{inbound}");
    }
    for unrelated in [
        "/?q=alacritty",
        "/?sort=name",
        "/?verified=1",
        "/?category=developer-tools",
        "/categories/developer-tools?sort=name",
        "/projects/unrelated-tool",
    ] {
        let (status, _, body) = send(&app, get(unrelated)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(!body.contains("href=\"/guides/rust-terminal-tools\""), "{unrelated}");
    }
}

#[tokio::test]
async fn standalone_guides_are_discoverable_canonical_and_cached_without_filter_leaks() {
    let (state, app, _dir) = app().await;
    for slug in ["zed", "lapce", "hurl", "difftastic", "gitui", "unrelated-tool"] {
        let (status, _, _) = send(&app, admin_json("POST", "/api/admin/projects", sample_project(slug))).await;
        assert_eq!(status, StatusCode::OK);
    }
    for (path, headline, published, projects) in [
        ("/guides/zed-vs-lapce", "Zed vs Lapce: choose by workflow", "2026-10-13", vec!["zed", "lapce"]),
        ("/guides/hurl-http-tests", "Hurl HTTP tests: catch a false green", "2026-10-25", vec!["hurl"]),
        (
            "/guides/difftastic-vs-git-diff",
            "Difftastic vs git diff: keep both views",
            "2026-10-26T21:38:00Z",
            vec!["difftastic", "gitui"],
        ),
    ] {
        let canonical = format!("http://test.local{path}");
        let (status, headers, body) = send(&app, get(path)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.matches("<h1>").count(), 1);
        assert!(body.contains(&format!("<h1>{headline}</h1>")));
        assert!(body.contains(&format!("<title>{headline} — Built with Rust</title>")));
        assert!(body.contains(&format!("rel=\"canonical\" href=\"{canonical}\"")));
        assert!(body.contains("property=\"og:type\" content=\"article\""));
        assert!(!body.contains("noindex"));
        if path == "/guides/zed-vs-lapce" {
            let image_path = state.asset_url("editor-choice.png");
            assert!(body.contains(&format!("property=\"og:image\" content=\"http://test.local{image_path}\"")));
            let (image_status, image_headers, _) = send(&app, get(&image_path)).await;
            assert_eq!(image_status, StatusCode::OK);
            assert_eq!(image_headers.get(header::CONTENT_TYPE).unwrap(), "image/png");
        }
        let start =
            body.find("<script type=\"application/ld+json\">").unwrap() + "<script type=\"application/ld+json\">".len();
        let end = body[start..].find("</script>").unwrap() + start;
        let schema: Value = serde_json::from_str(&body[start..end]).unwrap();
        let article = schema["@graph"].as_array().unwrap().iter().find(|node| node["@type"] == "Article").unwrap();
        assert_eq!(article["url"], canonical);
        assert_eq!(article["headline"], headline);
        assert_eq!(article["datePublished"], published);
        assert_eq!(article["dateModified"], published);
        for slug in &projects {
            assert!(body.contains(&format!("href=\"/projects/{slug}\"")));
            assert_eq!(send(&app, get(&format!("/projects/{slug}"))).await.0, StatusCode::OK);
        }
        assert!(state.cache.get(path).is_some());
        let etag = headers.get(header::ETAG).unwrap();
        let conditional = Request::get(path).header(header::IF_NONE_MATCH, etag).body(Body::empty()).unwrap();
        let (status, _, body) = send(&app, conditional).await;
        assert_eq!(status, StatusCode::NOT_MODIFIED);
        assert!(body.is_empty());
        let (_, _, sitemap) = send(&app, get("/sitemap.xml")).await;
        assert!(sitemap.contains(&format!("<url><loc>{canonical}</loc><lastmod>{published}</lastmod></url>")));
        assert!(sitemap.contains("<url><loc>http://test.local/</loc></url>"));
        let mut inbound = vec!["/guides".to_string()];
        inbound.extend(projects.iter().map(|slug| format!("/projects/{slug}")));
        for inbound in inbound {
            let (status, _, body) = send(&app, get(&inbound)).await;
            assert_eq!(status, StatusCode::OK);
            assert!(body.contains(&format!("href=\"{path}\"")), "{inbound}");
        }
        for unrelated in [
            "/?q=zed",
            "/?sort=name",
            "/?verified=1",
            "/?category=developer-tools",
            "/categories/developer-tools",
            "/categories/developer-tools?sort=name",
            "/projects/unrelated-tool",
        ] {
            let (status, _, body) = send(&app, get(unrelated)).await;
            assert_eq!(status, StatusCode::OK);
            // The canonical category now intentionally introduces the Hurl exercise;
            // filtered views and unrelated guides must still stay isolated.
            let category_hurl_entry = unrelated == "/categories/developer-tools" && path == "/guides/hurl-http-tests";
            assert_eq!(body.contains(&format!("href=\"{path}\"")), category_hurl_entry, "{unrelated}");
        }
        // Homepage links to the editorial index, not individual guide promotions.
        let homepage = send(&app, get("/")).await.2;
        assert!(!homepage.contains(&format!("href=\"{path}\"")));
        assert!(homepage.contains("href=\"/guides\""));
    }
    // The editorial list must be a real, independently cached route, not a catalog filter.
    let (status, headers, body) = send(&app, get("/guides")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("rel=\"canonical\" href=\"http://test.local/guides\""));
    assert!(body.contains("\"@type\":\"CollectionPage\""));
    assert_eq!(body.matches("<h1>").count(), 1);
    assert!(state.cache.get("/guides").is_some());
    let conditional =
        Request::get("/guides").header(header::IF_NONE_MATCH, &headers[header::ETAG]).body(Body::empty()).unwrap();
    assert_eq!(send(&app, conditional).await.0, StatusCode::NOT_MODIFIED);
}

#[tokio::test]
async fn directory_pagination_keeps_filters_and_never_loses_or_repeats_projects() {
    let (state, app, _dir) = app().await;
    let projects: Vec<_> = (0..31)
        .map(|i| {
            json!({
                "slug": format!("tool-{i:02}"), "name": format!("Tool {i:02}"),
                "tagline": "Rust pagination fixture", "website_url": "https://example.com",
                "category": "tools", "stack": ["tokio"], "license": "MIT & Apache",
                "verified": true, "published": true, "stars": 1200,
                "repo_url": "https://github.com/example/tool"
            })
        })
        .collect();
    assert_eq!(send(&app, admin_json("POST", "/api/admin/projects/bulk", json!(projects))).await.0, StatusCode::OK);
    let (_, _, homepage) = send(&app, get("/")).await;
    assert_eq!(card_slugs(&homepage).len(), 15);
    assert!(homepage.contains("aria-current=\"page\">1"));
    assert!(!homepage.contains("class=\"task-guide\""));
    let cache_size = state.cache.len();
    let filters =
        "q=Rust&category=tools&stack=tokio&license=MIT+%26+Apache&source=available&verified=1&stars=1000&sort=name";
    let mut seen = Vec::new();
    for (page, count) in [(1, 15), (2, 15), (3, 1)] {
        let (status, _, body) = send(&app, get(&format!("/?{filters}&page={page}"))).await;
        assert_eq!(status, StatusCode::OK);
        let slugs = card_slugs(&body);
        assert_eq!(slugs.len(), count);
        seen.extend(slugs.into_iter().map(str::to_owned));
        if page < 3 {
            assert!(
                body.contains(&format!("{}&amp;page={}#projects", filters.replace('&', "&amp;"), page + 1)),
                "filter state must survive Next"
            );
        }
    }
    assert_eq!(seen, (0..31).map(|i| format!("tool-{i:02}")).collect::<Vec<_>>());
    for page in ["0", "-1", "garbage", "999999999999999999999999999999999999"] {
        let (status, _, body) = send(&app, get(&format!("/?sort=name&page={page}"))).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(card_slugs(&body).first(), Some(&"tool-00"));
    }
    let (_, _, last) = send(&app, get("/?sort=name&page=99999")).await;
    assert_eq!(card_slugs(&last), ["tool-30"]);
    assert!(last.contains("aria-disabled=\"true\">Next"));
    let (_, _, empty) = send(&app, get("/?q=missing&page=2")).await;
    assert!(card_slugs(&empty).is_empty());
    assert!(!empty.contains("aria-label=\"Project pages\""));
    let (_, _, second) = send(&app, get("/?page=2")).await;
    assert!(second.contains("rel=\"canonical\" href=\"http://test.local/?page=2\""));
    assert_eq!(state.cache.len(), cache_size, "page query variants must not grow the default cache");
    assert_eq!(card_slugs(&send(&app, get("/")).await.2).len(), 15);
}
