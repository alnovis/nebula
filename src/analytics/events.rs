use serde::Deserialize;
use sqlx::PgPool;

/// A server-side page navigation event
pub struct PageEvent {
    pub session_id: String,
    pub path: String,
    pub prev_path: Option<String>,
    pub referrer: Option<String>,
    pub user_agent: Option<String>,
    pub ip_hash: String,
    pub country: Option<String>,
}

/// A client-side behavioral event payload (from JS beacon)
#[derive(Deserialize)]
pub struct ClientEventPayload {
    pub event_type: String,
    pub path: String,
    pub data: serde_json::Value,
}

/// Valid client event types
const VALID_EVENT_TYPES: &[&str] = &["scroll", "outbound_click", "visibility"];

impl ClientEventPayload {
    pub fn is_valid(&self) -> bool {
        VALID_EVENT_TYPES.contains(&self.event_type.as_str())
    }
}

impl PageEvent {
    /// Insert into PostgreSQL
    pub async fn insert(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO page_events (session_id, path, prev_path, referrer, user_agent, ip_hash, country)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&self.session_id)
        .bind(&self.path)
        .bind(&self.prev_path)
        .bind(&self.referrer)
        .bind(&self.user_agent)
        .bind(&self.ip_hash)
        .bind(&self.country)
        .execute(pool)
        .await?;
        Ok(())
    }
}

/// Insert a client event
pub async fn insert_client_event(
    pool: &PgPool,
    session_id: &str,
    path: &str,
    event_type: &str,
    event_data: &serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO client_events (session_id, path, event_type, event_data)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(session_id)
    .bind(path)
    .bind(event_type)
    .bind(event_data)
    .execute(pool)
    .await?;
    Ok(())
}
