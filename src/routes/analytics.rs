use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};

use crate::{
    analytics::{events, session},
    state::AppState,
    views,
};

/// POST /api/analytics/event
/// Receives client-side behavioral events (scroll, outbound_click, visibility)
pub async fn record_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<events::ClientEventPayload>,
) -> StatusCode {
    let sid = match session::extract_session_id(&headers) {
        Some(s) => s,
        None => return StatusCode::BAD_REQUEST,
    };

    if !payload.is_valid() {
        return StatusCode::BAD_REQUEST;
    }

    if views::is_bot(views::extract_user_agent(&headers).as_deref()) {
        return StatusCode::OK;
    }

    let pool = state.pool.clone();
    tokio::spawn(async move {
        let _ = events::insert_client_event(
            &pool,
            &sid,
            &payload.path,
            &payload.event_type,
            &payload.data,
        )
        .await;
    });

    StatusCode::NO_CONTENT
}
