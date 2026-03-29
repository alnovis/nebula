use axum::{
    body::Body,
    extract::State,
    http::{header, Request},
    middleware::Next,
    response::Response,
};

use crate::{
    analytics::{events::PageEvent, geoip, session},
    state::AppState,
    views,
};

/// Paths to skip for analytics tracking
fn should_skip(path: &str) -> bool {
    path.starts_with("/static/")
        || path.starts_with("/health")
        || path.starts_with("/api/analytics")
        || path.starts_with("/admin")
        || path.starts_with("/.well-known/")
        || path == "/favicon.ico"
        || path == "/robots.txt"
        || path == "/sitemap.xml"
        || path == "/rss.xml"
}

pub async fn analytics_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();

    if should_skip(&path) {
        return next.run(request).await;
    }

    let headers = request.headers().clone();
    let ua = views::extract_user_agent(&headers);

    if views::is_bot(ua.as_deref()) {
        return next.run(request).await;
    }

    let ip = views::extract_client_ip(&headers, None);
    let cf_country = headers
        .get("cf-ipcountry")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .filter(|s| s.len() == 2 && s != "XX" && s != "T1" && s != "ZZ");
    let referrer = headers
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let existing_sid = session::extract_session_id(&headers);
    let (sid, is_new) = match existing_sid {
        Some(s) => (s, false),
        None => (session::new_session_id(), true),
    };

    // Get prev_path from Redis session
    let prev_path = if !is_new {
        if let Some(ref redis) = state.redis {
            let mut conn = redis.clone();
            session::get_session(&mut conn, &sid)
                .await
                .and_then(|s| s.prev_path)
        } else {
            None
        }
    } else {
        None
    };

    // Execute the actual handler
    let mut response = next.run(request).await;

    // Set session cookie on response
    let cookie_value = session::session_cookie(&sid, state.config.is_production());
    if let Ok(hv) = cookie_value.parse() {
        response.headers_mut().append(header::SET_COOKIE, hv);
    }

    // Fire-and-forget: write event + update session
    let pool = state.pool.clone();
    let redis = state.redis.clone();
    let path_clone = path.clone();
    let ip_clone = ip.clone();
    let cf_country_clone = cf_country.clone();
    let ip_hash = ip.as_deref().map(views::hash_ip_daily).unwrap_or_default();
    let sid_clone = sid.clone();
    let referrer_clone = referrer.clone();
    let ua_clone = ua.clone();

    tokio::spawn(async move {
        // Country: prefer Cloudflare header, fallback to DB-IP lookup
        let country = if cf_country_clone.is_some() {
            cf_country_clone
        } else {
            match ip_clone.as_deref() {
                Some(raw_ip) => geoip::lookup_country(&pool, raw_ip).await,
                None => None,
            }
        };

        let event = PageEvent {
            session_id: sid_clone.clone(),
            path: path_clone.clone(),
            prev_path,
            referrer: referrer_clone.clone(),
            user_agent: ua_clone.clone(),
            ip_hash: ip_hash.clone(),
            country,
        };
        if let Err(e) = event.insert(&pool).await {
            tracing::warn!("Failed to insert page event: {}", e);
        }

        let _ = session::ensure_session_row(
            &pool,
            &sid_clone,
            &path_clone,
            referrer_clone.as_deref(),
            &ip_hash,
            ua_clone.as_deref(),
        )
        .await;

        if let Some(redis) = redis {
            let mut conn = redis.clone();
            let new_state = session::SessionState {
                prev_path: Some(path_clone),
            };
            let _ = session::update_session(&mut conn, &sid_clone, &new_state).await;
        }
    });

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_skip() {
        assert!(should_skip("/static/js/main.js"));
        assert!(should_skip("/health"));
        assert!(should_skip("/health/cdn"));
        assert!(should_skip("/api/analytics/event"));
        assert!(should_skip("/favicon.ico"));
        assert!(should_skip("/robots.txt"));
        assert!(should_skip("/sitemap.xml"));
        assert!(should_skip("/rss.xml"));

        assert!(!should_skip("/"));
        assert!(!should_skip("/blog"));
        assert!(!should_skip("/blog/my-post"));
        assert!(!should_skip("/projects"));
        assert!(!should_skip("/about"));
        assert!(!should_skip("/contact"));
    }
}
