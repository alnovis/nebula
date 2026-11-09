//! Visits left out of analytics: the owner's browsers and listed IP addresses.

use std::net::IpAddr;

use axum::http::HeaderMap;
use tracing::warn;

const OWNER_COOKIE: &str = "nb_owner=1";
const OWNER_COOKIE_TTL_SECS: u64 = 365 * 24 * 3600;

/// Set-Cookie value that marks a browser as the owner's
pub fn owner_cookie(secure: bool) -> String {
    format!(
        "{}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}",
        OWNER_COOKIE,
        OWNER_COOKIE_TTL_SECS,
        if secure { "; Secure" } else { "" }
    )
}

/// Parse a comma or whitespace separated list of IP addresses; invalid entries are skipped
pub fn parse_ips(list: &str) -> Vec<IpAddr> {
    list.split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .filter_map(|s| match s.parse() {
            Ok(ip) => Some(ip),
            Err(_) => {
                warn!("ANALYTICS_EXCLUDE_IPS: not an IP address, skipped: {}", s);
                None
            }
        })
        .collect()
}

/// A request is not tracked if it carries the owner cookie, or if any address of its
/// forwarding chain (X-Forwarded-For, X-Real-IP) is listed. Every hop is checked, so
/// requests proxied through a listed host are left out as well.
pub fn is_excluded(headers: &HeaderMap, ips: &[IpAddr]) -> bool {
    has_owner_cookie(headers) || chain_ips(headers).any(|ip| ips.contains(&ip))
}

fn has_owner_cookie(headers: &HeaderMap) -> bool {
    headers
        .get_all("cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .any(|c| c.trim() == OWNER_COOKIE)
}

fn chain_ips(headers: &HeaderMap) -> impl Iterator<Item = IpAddr> + '_ {
    ["x-forwarded-for", "x-real-ip"]
        .into_iter()
        .flat_map(move |name| headers.get_all(name).iter())
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .filter_map(|s| s.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (name, value) in pairs {
            h.append(*name, value.parse().unwrap());
        }
        h
    }

    #[test]
    fn test_parse_ips() {
        let ips = parse_ips("1.2.3.4, 5.6.7.8\n2001:db8::1,not-an-ip,,");
        assert_eq!(
            ips,
            vec![
                "1.2.3.4".parse::<IpAddr>().unwrap(),
                "5.6.7.8".parse().unwrap(),
                "2001:db8::1".parse().unwrap(),
            ]
        );
    }

    #[test]
    fn test_owner_cookie_excludes() {
        let h = headers(&[("cookie", "nb_sid=abc; nb_owner=1")]);
        assert!(is_excluded(&h, &[]));
    }

    #[test]
    fn test_other_cookie_value_not_excluded() {
        let h = headers(&[("cookie", "nb_owner=0; nb_sid=abc")]);
        assert!(!is_excluded(&h, &[]));
    }

    #[test]
    fn test_listed_ip_anywhere_in_chain_excludes() {
        let ips = parse_ips("10.0.0.7");
        assert!(is_excluded(
            &headers(&[("x-forwarded-for", "10.0.0.7")]),
            &ips
        ));
        assert!(is_excluded(
            &headers(&[("x-forwarded-for", "203.0.113.9, 10.0.0.7")]),
            &ips
        ));
        assert!(is_excluded(
            &headers(&[
                ("x-forwarded-for", "203.0.113.9"),
                ("x-real-ip", "10.0.0.7")
            ]),
            &ips
        ));
    }

    #[test]
    fn test_unlisted_ip_not_excluded() {
        let ips = parse_ips("10.0.0.7");
        let h = headers(&[
            ("x-forwarded-for", "203.0.113.9, 203.0.113.9"),
            ("x-real-ip", "203.0.113.9"),
        ]);
        assert!(!is_excluded(&h, &ips));
        assert!(!is_excluded(&h, &[]));
    }
}
