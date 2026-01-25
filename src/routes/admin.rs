use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct ReloadQuery {
    secret: String,
}

/// Reload content from filesystem
/// Usage: POST /admin/reload?secret=YOUR_SECRET
pub async fn reload_content(
    State(state): State<AppState>,
    Query(query): Query<ReloadQuery>,
) -> impl IntoResponse {
    // Check if admin secret is configured
    let Some(admin_secret) = &state.config.admin_secret else {
        return (StatusCode::FORBIDDEN, "Admin access not configured");
    };

    // Validate secret
    if query.secret != *admin_secret {
        return (StatusCode::FORBIDDEN, "Invalid secret");
    }

    // Reload content
    match state.reload_content().await {
        Ok(_) => {
            tracing::info!("Content reloaded successfully");
            (StatusCode::OK, "Content reloaded successfully")
        }
        Err(e) => {
            tracing::error!("Failed to reload content: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to reload content",
            )
        }
    }
}
