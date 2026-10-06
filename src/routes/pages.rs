//! Public HTML pages. Everything here is read-only and cached.

use super::{AppError, cached_html, html_now, page};
use crate::SharedState;
use crate::db;
use crate::views::pages as v;
use crate::views::project as vp;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct HomeQuery {
    pub q: Option<String>,
}

pub async fn home(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Query(query): Query<HomeQuery>,
) -> Result<Response, AppError> {
    let q = query.q.as_deref().map(str::trim).filter(|q| !q.is_empty());
    if let Some(q) = q {
        // Search results are not cached: the key space is unbounded.
        let q = q.chars().take(100).collect::<String>();
        let projects = db::list_projects(&state.pool, None, Some(&q)).await?;
        let categories = db::categories_with_counts(&state.pool).await?;
        let total = db::project_count(&state.pool).await?;
        state.posthog.capture("search", "server", serde_json::json!({ "query": q, "results": projects.len() }));
        let body = v::home(v::HomeData {
            projects: &projects,
            categories: &categories,
            total,
            query: Some(&q),
            active_category: None,
        });
        return Ok(html_now(page(&state, "Search", "Search projects built with Rust", "/", body)));
    }
    cached_html(&state, &headers, "/".into(), || async {
        let projects = db::list_projects(&state.pool, None, None).await?;
        let categories = db::categories_with_counts(&state.pool).await?;
        let total = db::project_count(&state.pool).await?;
        let body = v::home(v::HomeData {
            projects: &projects,
            categories: &categories,
            total,
            query: None,
            active_category: None,
        });
        Ok(page(&state, "Home", "A curated directory of websites, apps and tools built with Rust.", "/", body))
    })
    .await
}

pub async fn categories(State(state): State<SharedState>, headers: HeaderMap) -> Result<Response, AppError> {
    cached_html(&state, &headers, "/categories".into(), || async {
        let cats = db::categories_with_counts(&state.pool).await?;
        Ok(page(&state, "Categories", "Browse Rust-built projects by category.", "/categories", v::categories(&cats)))
    })
    .await
}

pub async fn category(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> Result<Response, AppError> {
    let path = format!("/categories/{slug}");
    // Cache first: a hit must not touch the database.
    if let Some(hit) = state.cache.get(&path) {
        return Ok(hit.respond(&headers));
    }
    let Some(cat) = db::category_by_slug(&state.pool, &slug).await? else {
        return Ok(not_found(State(state)).await);
    };
    cached_html(&state, &headers, path.clone(), || async {
        let projects = db::list_projects(&state.pool, Some(cat.id), None).await?;
        let categories = db::categories_with_counts(&state.pool).await?;
        let total = db::project_count(&state.pool).await?;
        let active = categories.iter().find(|c| c.id == cat.id);
        let desc = format!("{} built with Rust: {} projects.", cat.name, projects.len());
        let body = v::home(v::HomeData {
            projects: &projects,
            categories: &categories,
            total,
            query: None,
            active_category: active,
        });
        Ok(page(&state, &cat.name, &desc, &path, body))
    })
    .await
}

pub async fn project(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> Result<Response, AppError> {
    let path = format!("/projects/{slug}");
    if let Some(hit) = state.cache.get(&path) {
        return Ok(hit.respond(&headers));
    }
    let Some(p) = db::project_by_slug(&state.pool, &slug).await?.filter(|p| p.published != 0) else {
        return Ok(not_found(State(state)).await);
    };
    cached_html(&state, &headers, path.clone(), || async {
        let related = db::related_projects(&state.pool, &p).await?;
        let desc = format!("{}: {} Built with Rust.", p.name, p.tagline);
        Ok(page(&state, &p.name, &desc, &path, vp::show(&p, &related)))
    })
    .await
}

pub async fn submit(State(state): State<SharedState>, headers: HeaderMap) -> Result<Response, AppError> {
    cached_html(&state, &headers, "/submit".into(), || async {
        Ok(page(
            &state,
            "Submit a site",
            "Submit a website, app or tool built with Rust for review.",
            "/submit",
            v::submit_form(None, ""),
        ))
    })
    .await
}

pub async fn feature(State(state): State<SharedState>, headers: HeaderMap) -> Result<Response, AppError> {
    cached_html(&state, &headers, "/feature".into(), || async {
        let projects = db::list_projects(&state.pool, None, None).await?;
        let body = v::feature(&projects, state.cfg.payments_enabled(), state.cfg.feature_days, None);
        Ok(page(
            &state,
            "Feature your project",
            "Pin your Rust project to the top of Built with Rust.",
            "/feature",
            body,
        ))
    })
    .await
}

#[derive(Deserialize)]
pub struct SuccessQuery {
    pub session_id: Option<String>,
}

pub async fn feature_success(
    State(state): State<SharedState>,
    Query(q): Query<SuccessQuery>,
) -> Result<Response, AppError> {
    let project = match q.session_id.as_deref() {
        Some(sid) => match db::payment_project_slug(&state.pool, sid).await? {
            Some(slug) => db::project_by_slug(&state.pool, &slug).await?,
            None => None,
        },
        None => None,
    };
    let body = v::feature_success(project.as_ref());
    Ok(html_now(page(&state, "Payment complete", "Your project is featured.", "/feature/success", body)))
}

pub async fn feature_cancel(State(state): State<SharedState>) -> Response {
    html_now(page(&state, "Payment cancelled", "No charge was made.", "/feature/cancel", v::feature_cancel()))
}

pub async fn not_found(State(state): State<SharedState>) -> Response {
    let mut resp = html_now(page(&state, "Not found", "Page not found.", "/404", v::not_found()));
    *resp.status_mut() = StatusCode::NOT_FOUND;
    resp.into_response()
}
