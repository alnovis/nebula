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

/// Check if referrer is a raw IP address (bot/scanner indicator).
/// Matches patterns like http://1.2.3.4/, http://1.2.3.4:443/, etc.
fn is_ip_referrer(referrer: &str) -> bool {
    let after_scheme = referrer
        .strip_prefix("http://")
        .or_else(|| referrer.strip_prefix("https://"))
        .unwrap_or(referrer);
    let host_port = after_scheme.split('/').next().unwrap_or("");
    // IPv6 bracket notation: http://[::1]:8080/
    if host_port.starts_with('[') {
        return true;
    }
    // IPv4: strip port, parse
    let host = host_port.split(':').next().unwrap_or("");
    host.parse::<std::net::Ipv4Addr>().is_ok()
}

/// Only track known application routes (whitelist approach).
/// Scanners hitting random paths (/.env, /phpinfo.php, /actuator, etc.) are ignored.
fn should_track(path: &str) -> bool {
    path == "/"
        || path == "/blog"
        || path.starts_with("/blog/")
        || path == "/projects"
        || path.starts_with("/projects/")
        || path == "/about"
        || path == "/resume"
        || path == "/resume/print"
        || path == "/contact"
}

pub async fn analytics_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();

    if !should_track(&path) {
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
        .map(|s| s.to_string())
        .filter(|s| !is_ip_referrer(s));

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
    fn test_should_track() {
        // Known routes — tracked
        assert!(should_track("/"));
        assert!(should_track("/blog"));
        assert!(should_track("/blog/my-post"));
        assert!(should_track("/projects"));
        assert!(should_track("/projects/ircraft"));
        assert!(should_track("/about"));
        assert!(should_track("/contact"));
        assert!(should_track("/resume"));
        assert!(should_track("/resume/print"));
        assert!(should_track("/blog/tag/rust"));

        // Infrastructure/scanner paths — not tracked
        assert!(!should_track("/static/js/main.js"));
        assert!(!should_track("/health"));
        assert!(!should_track("/api/analytics/event"));
        assert!(!should_track("/admin/analytics"));
        assert!(!should_track("/favicon.ico"));
        assert!(!should_track("/robots.txt"));
        assert!(!should_track("/.env"));
        assert!(!should_track("/actuator/gateway/routes"));
        assert!(!should_track("/phpinfo.php"));
        assert!(!should_track("/wp-login.php"));
    }
}
