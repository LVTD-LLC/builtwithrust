//! Embedded static assets, pre-compressed once and served from the cache.

use super::AppError;
use crate::{Assets, SharedState};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};

pub async fn serve(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(path): Path<String>,
) -> Result<Response, AppError> {
    let key = format!("asset:{path}");
    if let Some(hit) = state.cache.get(&key) {
        return Ok(hit.respond(&headers));
    }
    let Some(file) = Assets::get(&path) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let mime = mime_guess::from_path(&path).first_or_octet_stream().to_string();
    let entry = state.cache.insert(
        key,
        bytes::Bytes::copy_from_slice(&file.data),
        &mime,
        "public, max-age=31536000, immutable",
    );
    Ok(entry.respond(&headers))
}
