use super::AppError;
use crate::{SharedState, db};
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use std::process::Stdio;
use tokio::{process::Command, sync::Semaphore};

static RENDERS: Semaphore = Semaphore::const_new(2);

pub async fn get(State(state): State<SharedState>, Path(slug): Path<String>) -> Result<Response, AppError> {
    let Some(project) = db::project_by_slug(&state.pool, &slug).await?.filter(|p| p.published != 0) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let _permit = RENDERS.acquire().await?;
    let output = Command::new("python3")
        .args(["-c", include_str!("card.py")])
        .arg(project.name)
        .arg(project.tagline)
        .arg(project.category_name.unwrap_or_default())
        .stdin(Stdio::null())
        .kill_on_drop(true)
        .output()
        .await?;
    if !output.status.success() {
        return Err(anyhow::anyhow!("social card rendering failed").into());
    }
    Ok(([(header::CONTENT_TYPE, "image/png"), (header::CACHE_CONTROL, "public, max-age=3600")], output.stdout)
        .into_response())
}
