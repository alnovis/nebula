use axum::http::HeaderMap;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

const SESSION_TTL_SECS: u64 = 1800;
const SESSION_COOKIE_NAME: &str = "nb_sid";
const SESSION_REDIS_PREFIX: &str = "analytics:session:";

#[derive(Serialize, Deserialize, Default)]
pub struct SessionState {
    pub prev_path: Option<String>,
}

/// Extract session ID from Cookie header
pub fn extract_session_id(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    cookie_header
        .split(';')
        .map(|s| s.trim())
        .find(|s| s.starts_with(SESSION_COOKIE_NAME))
        .and_then(|s| s.strip_prefix(SESSION_COOKIE_NAME))
        .and_then(|s| s.strip_prefix('='))
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
}

/// Generate a new session ID
pub fn new_session_id() -> String {
    Uuid::new_v4().simple().to_string()
}

/// Build Set-Cookie header value
pub fn session_cookie(sid: &str, secure: bool) -> String {
    format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}",
        SESSION_COOKIE_NAME,
        sid,
        SESSION_TTL_SECS,
        if secure { "; Secure" } else { "" }
    )
}

/// Get session state from Redis
pub async fn get_session(redis: &mut ConnectionManager, sid: &str) -> Option<SessionState> {
    let key = format!("{}{}", SESSION_REDIS_PREFIX, sid);
    let raw: Option<String> = redis.get(&key).await.ok()?;
    raw.and_then(|s| serde_json::from_str(&s).ok())
}

/// Update session state in Redis (set prev_path, refresh TTL)
pub async fn update_session(
    redis: &mut ConnectionManager,
    sid: &str,
    state: &SessionState,
) -> anyhow::Result<()> {
    let key = format!("{}{}", SESSION_REDIS_PREFIX, sid);
    let json = serde_json::to_string(state)?;
    let _: () = redis.set_ex(&key, &json, SESSION_TTL_SECS).await?;
    Ok(())
}

/// Create or update session row in PostgreSQL
pub async fn ensure_session_row(
    pool: &PgPool,
    sid: &str,
    first_path: &str,
    first_referrer: Option<&str>,
    ip_hash: &str,
    user_agent: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO analytics_sessions (id, first_path, first_referrer, ip_hash, user_agent)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (id) DO UPDATE SET last_seen_at = NOW()",
    )
    .bind(sid)
    .bind(first_path)
    .bind(first_referrer)
    .bind(ip_hash)
    .bind(user_agent)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn test_extract_session_id() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "cookie",
            HeaderValue::from_static("nb_sid=abc123; other=val"),
        );
        assert_eq!(extract_session_id(&headers), Some("abc123".to_string()));
    }

    #[test]
    fn test_extract_session_id_missing() {
        let headers = HeaderMap::new();
        assert_eq!(extract_session_id(&headers), None);
    }

    #[test]
    fn test_extract_session_id_no_match() {
        let mut headers = HeaderMap::new();
        headers.insert("cookie", HeaderValue::from_static("other=val"));
        assert_eq!(extract_session_id(&headers), None);
    }

    #[test]
    fn test_new_session_id_format() {
        let sid = new_session_id();
        assert_eq!(sid.len(), 32);
        assert!(sid.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_session_cookie_dev() {
        let cookie = session_cookie("abc123", false);
        assert!(cookie.contains("nb_sid=abc123"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Lax"));
        assert!(!cookie.contains("Secure"));
    }

    #[test]
    fn test_session_cookie_prod() {
        let cookie = session_cookie("abc123", true);
        assert!(cookie.contains("; Secure"));
    }
}
