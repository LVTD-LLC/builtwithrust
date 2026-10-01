//! Admin JSON API. Authenticated with `Authorization: Bearer $ADMIN_TOKEN`.
//! This is how agents add and update listings; see AGENTS.md for examples.

use super::{AppError, bad_request};
use crate::SharedState;
use crate::db::{self, ProjectInput};
use axum::extract::{Path, Query, Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use subtle::ConstantTimeEq;

pub fn router(state: SharedState) -> Router<SharedState> {
    Router::new()
        .route("/projects", get(list_projects).post(upsert_project))
        .route("/projects/bulk", post(bulk_upsert))
        .route("/projects/{slug}", delete(delete_project))
        .route("/projects/{slug}/feature", post(feature_project))
        .route("/submissions", get(list_submissions))
        .route("/submissions/{id}/review", post(review_submission))
        .route("/cache/purge", post(purge_cache))
        .route("/stats", get(stats))
        .route_layer(middleware::from_fn_with_state(state, require_admin))
}

async fn require_admin(State(state): State<SharedState>, req: Request, next: Next) -> Response {
    let Some(expected) = &state.cfg.admin_token else {
        return (StatusCode::SERVICE_UNAVAILABLE, "admin API disabled: ADMIN_TOKEN not set").into_response();
    };
    let presented = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .unwrap_or("");
    let ok: bool = presented.as_bytes().ct_eq(expected.as_bytes()).into();
    if !ok {
        return (StatusCode::UNAUTHORIZED, "invalid or missing bearer token").into_response();
    }
    next.run(req).await
}

async fn list_projects(State(state): State<SharedState>) -> Result<Response, AppError> {
    let projects = db::all_projects(&state.pool).await?;
    Ok(Json(projects).into_response())
}

async fn upsert_project(
    State(state): State<SharedState>,
    Json(input): Json<ProjectInput>,
) -> Result<Response, AppError> {
    let project = db::upsert_project(&state.pool, &input).await.map_err(|e| bad_request(e.to_string()))?;
    state.cache.purge();
    state.posthog.capture("project_upserted", "admin", json!({ "slug": project.slug }));
    Ok((StatusCode::OK, Json(project)).into_response())
}

async fn bulk_upsert(
    State(state): State<SharedState>,
    Json(inputs): Json<Vec<ProjectInput>>,
) -> Result<Response, AppError> {
    let mut slugs = Vec::with_capacity(inputs.len());
    let mut errors = Vec::new();
    for input in &inputs {
        match db::upsert_project(&state.pool, input).await {
            Ok(p) => slugs.push(p.slug),
            Err(e) => errors.push(json!({ "name": input.name, "error": e.to_string() })),
        }
    }
    state.cache.purge();
    state.posthog.capture("projects_bulk_upserted", "admin", json!({ "count": slugs.len(), "errors": errors.len() }));
    Ok(Json(json!({ "upserted": slugs, "errors": errors })).into_response())
}

async fn delete_project(State(state): State<SharedState>, Path(slug): Path<String>) -> Result<Response, AppError> {
    let removed = db::delete_project(&state.pool, &slug).await?;
    state.cache.purge();
    Ok(if removed { StatusCode::NO_CONTENT } else { StatusCode::NOT_FOUND }.into_response())
}

#[derive(Deserialize)]
struct FeatureBody {
    /// Days to add to the featured window. Defaults to FEATURE_DAYS.
    days: Option<i64>,
}

async fn feature_project(
    State(state): State<SharedState>,
    Path(slug): Path<String>,
    body: Option<Json<FeatureBody>>,
) -> Result<Response, AppError> {
    let Some(project) = db::project_by_slug(&state.pool, &slug).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let days = body.and_then(|b| b.days).unwrap_or(state.cfg.feature_days);
    if !(1..=3650).contains(&days) {
        return Err(bad_request("days must be between 1 and 3650"));
    }
    db::extend_featured(&state.pool, project.id, days).await?;
    state.cache.purge();
    let project = db::project_by_slug(&state.pool, &slug).await?;
    Ok(Json(project).into_response())
}

#[derive(Deserialize)]
struct SubmissionsQuery {
    /// `pending` (default), `approved`, `rejected`, or `all`.
    status: Option<String>,
}

async fn list_submissions(
    State(state): State<SharedState>,
    Query(q): Query<SubmissionsQuery>,
) -> Result<Response, AppError> {
    let status = match q.status.as_deref() {
        None => Some("pending"),
        Some("all") => None,
        Some(s @ ("pending" | "approved" | "rejected")) => Some(s),
        Some(_) => return Err(bad_request("status must be pending, approved, rejected or all")),
    };
    let subs = db::list_submissions(&state.pool, status).await?;
    Ok(Json(subs).into_response())
}

#[derive(Deserialize)]
struct ReviewBody {
    /// `approved` or `rejected`.
    status: String,
    /// When approving, optionally create/update the listing in the same call.
    #[serde(default)]
    project: Option<ProjectInput>,
}

async fn review_submission(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
    Json(body): Json<ReviewBody>,
) -> Result<Response, AppError> {
    if body.status != "approved" && body.status != "rejected" {
        return Err(bad_request("status must be approved or rejected"));
    }
    let Some(sub) = db::get_submission(&state.pool, id).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let mut project = None;
    if body.status == "approved"
        && let Some(input) = &body.project
    {
        project = Some(db::upsert_project(&state.pool, input).await.map_err(|e| bad_request(e.to_string()))?);
        state.cache.purge();
    }
    let project_id = project.as_ref().map(|p| p.id).or(sub.project_id);
    db::review_submission(&state.pool, id, &body.status, project_id).await?;
    state.posthog.capture(
        "submission_reviewed",
        "admin",
        json!({ "submission_id": id, "status": body.status, "project": project.as_ref().map(|p| &p.slug) }),
    );
    let sub = db::get_submission(&state.pool, id).await?;
    Ok(Json(json!({ "submission": sub, "project": project })).into_response())
}

async fn purge_cache(State(state): State<SharedState>) -> Response {
    let before = state.cache.len();
    state.cache.purge();
    Json(json!({ "purged": before })).into_response()
}

async fn stats(State(state): State<SharedState>) -> Result<Response, AppError> {
    let projects = db::project_count(&state.pool).await?;
    let pending = db::list_submissions(&state.pool, Some("pending")).await?.len();
    Ok(Json(json!({
        "projects": projects,
        "pending_submissions": pending,
        "cache_entries": state.cache.len(),
        "payments_enabled": state.cfg.payments_enabled(),
        "posthog_enabled": state.posthog.enabled(),
    }))
    .into_response())
}
