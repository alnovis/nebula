//! Views counter service using Redis for storage.
//!
//! Tracks unique page views by hashing visitor IPs and filtering bots.

use axum::http::HeaderMap;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use sha2::{Digest, Sha256};
use std::net::SocketAddr;

/// Content type for views tracking
#[derive(Debug, Clone, Copy)]
pub enum ContentType {
    Post,
    Project,
}

impl ContentType {
    fn as_str(&self) -> &'static str {
        match self {
            ContentType::Post => "post",
            ContentType::Project => "project",
        }
    }
}

/// Bot detection patterns in User-Agent strings
const BOT_PATTERNS: &[&str] = &[
    "bot",
    "crawler",
    "spider",
    "slurp",
    "mediapartners",
    "facebookexternalhit",
    "linkedinbot",
    "twitterbot",
    "whatsapp",
    "telegram",
    "curl",
    "wget",
    "python",
    "go-http-client",
    "java/",
    "apache-httpclient",
    "httpx",
    "axios",
    "node-fetch",
    "googlebot",
    "bingbot",
    "yandexbot",
    "duckduckbot",
    "baiduspider",
    "sogou",
    "exabot",
    "ahrefsbot",
    "semrushbot",
    "dotbot",
    "petalbot",
    "mj12bot",
];

/// Extract client IP from request headers or socket address.
/// Priority: X-Forwarded-For > X-Real-IP > socket peer
pub fn extract_client_ip(headers: &HeaderMap, peer: Option<SocketAddr>) -> Option<String> {
    // Try X-Forwarded-For first (may contain multiple IPs, take first)
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(value) = xff.to_str() {
            if let Some(first_ip) = value.split(',').next() {
                let ip = first_ip.trim();
                if !ip.is_empty() {
                    return Some(ip.to_string());
                }
            }
        }
    }

    // Try X-Real-IP
    if let Some(xri) = headers.get("x-real-ip") {
        if let Ok(value) = xri.to_str() {
            let ip = value.trim();
            if !ip.is_empty() {
                return Some(ip.to_string());
            }
        }
    }

    // Fall back to peer socket address
    peer.map(|addr| addr.ip().to_string())
}

/// Extract User-Agent from headers
pub fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Views counter service
pub struct ViewsService {
    redis: ConnectionManager,
}

impl ViewsService {
    /// Create a new views service with Redis connection
    pub fn new(redis: ConnectionManager) -> Self {
        Self { redis }
    }

    /// Record a page view. Returns true if this was a new unique visitor.
    pub async fn record_view(
        &self,
        content_type: ContentType,
        slug: &str,
        ip: &str,
        user_agent: Option<&str>,
    ) -> anyhow::Result<bool> {
        // Skip bots
        if self.is_bot(user_agent) {
            return Ok(false);
        }

        let ip_hash = Self::hash_ip(ip);
        let key_visitors = format!("views:{}:{}:visitors", content_type.as_str(), slug);
        let key_count = format!("views:{}:{}:count", content_type.as_str(), slug);

        let mut conn = self.redis.clone();

        // SADD returns number of elements added (1 if new, 0 if exists)
        let added: i32 = conn.sadd(&key_visitors, &ip_hash).await?;

        if added == 1 {
            // New unique visitor - increment counter
            let _: () = conn.incr(&key_count, 1).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get the view count for a page
    pub async fn get_count(&self, content_type: ContentType, slug: &str) -> anyhow::Result<u64> {
        let key = format!("views:{}:{}:count", content_type.as_str(), slug);
        let mut conn = self.redis.clone();

        let count: Option<u64> = conn.get(&key).await?;
        Ok(count.unwrap_or(0))
    }

    /// Get view counts for multiple pages (batch)
    pub async fn get_counts(
        &self,
        content_type: ContentType,
        slugs: &[&str],
    ) -> anyhow::Result<Vec<u64>> {
        if slugs.is_empty() {
            return Ok(vec![]);
        }

        let keys: Vec<String> = slugs
            .iter()
            .map(|slug| format!("views:{}:{}:count", content_type.as_str(), slug))
            .collect();

        let mut conn = self.redis.clone();
        let counts: Vec<Option<u64>> = redis::cmd("MGET").arg(&keys).query_async(&mut conn).await?;

        Ok(counts.into_iter().map(|c| c.unwrap_or(0)).collect())
    }

    /// Check if User-Agent indicates a bot
    fn is_bot(&self, user_agent: Option<&str>) -> bool {
        let Some(ua) = user_agent else {
            // No User-Agent is suspicious, treat as bot
            return true;
        };

        let ua_lower = ua.to_lowercase();
        BOT_PATTERNS
            .iter()
            .any(|pattern| ua_lower.contains(pattern))
    }

    /// Hash IP address for privacy (we don't store raw IPs)
    fn hash_ip(ip: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(ip.as_bytes());
        // Add salt to prevent rainbow table attacks
        hasher.update(b"nebula-views-salt-2024");
        let result = hasher.finalize();
        // Use first 16 bytes (32 hex chars) - enough for uniqueness
        hex::encode(&result[..16])
    }
}

/// Format view count for display
/// - <100: exact number
/// - 100-999: exact number
/// - 1000-9999: "1.2k"
/// - 10000+: "12k"
/// - 1000000+: "1.2M"
pub fn format_count(count: u64) -> String {
    match count {
        0 => "0".to_string(),
        1..=99 => count.to_string(),
        100..=999 => count.to_string(),
        1000..=9999 => {
            let k = count as f64 / 1000.0;
            let rounded = (k * 10.0).round() / 10.0;
            if rounded.fract().abs() < 0.01 {
                format!("{}k", rounded as u64)
            } else {
                format!("{:.1}k", rounded)
            }
        }
        10000..=999999 => format!("{}k", count / 1000),
        _ => {
            let m = count as f64 / 1_000_000.0;
            format!("{:.1}M", m)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_count() {
        assert_eq!(format_count(0), "0");
        assert_eq!(format_count(1), "1");
        assert_eq!(format_count(99), "99");
        assert_eq!(format_count(100), "100");
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(1000), "1k");
        assert_eq!(format_count(1500), "1.5k");
        assert_eq!(format_count(1234), "1.2k");
        assert_eq!(format_count(9999), "10k");
        assert_eq!(format_count(10000), "10k");
        assert_eq!(format_count(15000), "15k");
        assert_eq!(format_count(100000), "100k");
        assert_eq!(format_count(1000000), "1.0M");
        assert_eq!(format_count(1500000), "1.5M");
    }

    #[test]
    fn test_bot_patterns() {
        // Verify the patterns are reasonable
        assert!(BOT_PATTERNS.contains(&"bot"));
        assert!(BOT_PATTERNS.contains(&"googlebot"));
        assert!(BOT_PATTERNS.contains(&"curl"));
    }

    #[test]
    fn test_hash_ip() {
        let hash1 = ViewsService::hash_ip("192.168.1.1");
        let hash2 = ViewsService::hash_ip("192.168.1.1");
        let hash3 = ViewsService::hash_ip("192.168.1.2");

        // Same IP should produce same hash
        assert_eq!(hash1, hash2);
        // Different IP should produce different hash
        assert_ne!(hash1, hash3);
        // Hash should be 32 hex chars (16 bytes)
        assert_eq!(hash1.len(), 32);
    }
}
