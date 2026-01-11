use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    database: &'static str,
    posts_count: usize,
    projects_count: usize,
}

pub async fn check(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    // Check database connection
    let db_status = sqlx::query("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .map(|_| "ok")
        .unwrap_or("error");

    let content = state.content.read().await;

    Ok(Json(HealthResponse {
        status: "ok",
        database: db_status,
        posts_count: content.posts.len(),
        projects_count: content.projects.len(),
    }))
}
