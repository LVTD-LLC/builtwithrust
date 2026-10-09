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

pub async fn home(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Query(mut browse): Query<crate::directory::Browse>,
) -> Result<Response, AppError> {
    browse.normalize();
    if !browse.is_default() {
        return Ok(html_now(directory_page(&state, &browse, "/").await?));
    }
    cached_html(&state, &headers, "/".into(), || directory_page(&state, &browse, "/")).await
}

async fn directory_page(
    state: &SharedState,
    browse: &crate::directory::Browse,
    path: &str,
) -> Result<maud::Markup, AppError> {
    let mut projects = db::list_projects(&state.pool, None, None).await?;
    let total = projects.len() as i64;
    let facets = crate::directory::Facets::from_projects(&projects);
    let categories = db::categories_with_counts(&state.pool).await?;
    browse.apply(&mut projects);
    if !browse.q.is_empty() {
        state.posthog.capture("search", "server", serde_json::json!({ "query": browse.q, "results": projects.len() }));
    }
    let active = categories.iter().find(|c| c.slug == browse.category);
    let show_guide = path == "/categories/developer-tools"
        && *browse == crate::directory::Browse { category: "developer-tools".into(), ..Default::default() };
    let body = v::home(v::HomeData {
        developer_guide_site_url: show_guide.then_some(state.cfg.site_url.as_str()),
        projects: &projects,
        categories: &categories,
        total,
        newsletter_enabled: state.cfg.newsletter.is_some(),
        query: (!browse.q.is_empty()).then_some(browse.q.as_str()),
        active_category: active,
        browse,
        facets: &facets,
    });
    let title = active.map_or("Explore projects", |c| c.name.as_str());
    let description = active.map_or_else(
        || "Discover websites, apps and tools built with Rust.".to_string(),
        |c| format!("Explore {} built with Rust. Compare projects, GitHub stars, licenses and crates, then visit their websites or source repositories.", c.name),
    );
    Ok(page(state, title, &description, path, body))
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
    Query(mut browse): Query<crate::directory::Browse>,
) -> Result<Response, AppError> {
    browse.normalize();
    // The route owns the category; a conflicting query parameter cannot escape it.
    browse.category.clear();
    let filtered = !browse.is_default();
    let path = format!("/categories/{slug}");
    if !filtered && let Some(hit) = state.cache.get(&path) {
        return Ok(hit.respond(&headers));
    }
    if db::category_by_slug(&state.pool, &slug).await?.is_none() {
        return Ok(not_found(State(state)).await);
    }
    browse.category = slug;
    if filtered {
        return Ok(html_now(directory_page(&state, &browse, &path).await?));
    }
    cached_html(&state, &headers, path.clone(), || directory_page(&state, &browse, &path)).await
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
