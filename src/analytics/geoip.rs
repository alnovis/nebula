use sqlx::PgPool;
use std::io::Read;
use std::time::Duration;
use tokio::time;

/// Lookup country by IP address from geoip_ranges table
pub async fn lookup_country(pool: &PgPool, ip: &str) -> Option<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT country FROM geoip_ranges WHERE network >>= $1::inet LIMIT 1",
    )
    .bind(ip)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

/// Start nightly GeoIP update job
pub fn start_updater(pool: PgPool, client: reqwest::Client, db_ip_url: Option<String>) {
    let url = match db_ip_url {
        Some(u) if !u.is_empty() => u,
        _ => current_dbip_url(),
    };

    tokio::spawn(async move {
        // Initial load on startup (after short delay)
        time::sleep(Duration::from_secs(10)).await;
        if let Err(e) = update_geoip_data(&pool, &client, &url).await {
            tracing::error!("Initial GeoIP update failed: {}", e);
        }

        // Then run every 24 hours
        let mut interval = time::interval(Duration::from_secs(86400));
        interval.tick().await;
        loop {
            interval.tick().await;
            // Refresh URL in case month changed
            let fresh_url = current_dbip_url();
            tracing::info!("Running nightly GeoIP update");
            if let Err(e) = update_geoip_data(&pool, &client, &fresh_url).await {
                tracing::error!("GeoIP update failed: {}", e);
            }
        }
    });
}

/// Build current month's DB-IP download URL
fn current_dbip_url() -> String {
    let now = chrono::Utc::now();
    format!(
        "https://download.db-ip.com/free/dbip-country-lite-{}-{:02}.csv.gz",
        now.format("%Y"),
        now.format("%m")
    )
}

/// Download DB-IP CSV and refresh geoip_ranges table
async fn update_geoip_data(
    pool: &PgPool,
    client: &reqwest::Client,
    url: &str,
) -> anyhow::Result<()> {
    tracing::info!("Downloading DB-IP data: {}", url);

    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        anyhow::bail!("DB-IP download failed: HTTP {}", response.status());
    }

    let bytes = response.bytes().await?;
    let csv_text = decompress_gzip(&bytes)?;
    let entries = parse_dbip_csv(&csv_text);

    if entries.is_empty() {
        tracing::warn!("No GeoIP entries parsed, skipping update");
        return Ok(());
    }

    tracing::info!(
        "Refreshing geoip_ranges table with {} entries",
        entries.len()
    );

    let mut tx = pool.begin().await?;

    sqlx::query("TRUNCATE geoip_ranges")
        .execute(&mut *tx)
        .await?;

    for chunk in entries.chunks(5000) {
        let networks: Vec<&str> = chunk.iter().map(|(n, _)| n.as_str()).collect();
        let countries: Vec<&str> = chunk.iter().map(|(_, c)| c.as_str()).collect();

        sqlx::query(
            "INSERT INTO geoip_ranges (network, country)
             SELECT unnest($1::cidr[]), unnest($2::varchar[])",
        )
        .bind(&networks)
        .bind(&countries)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    tracing::info!("GeoIP update complete: {} entries loaded", entries.len());
    Ok(())
}

/// Decompress gzip bytes to string
fn decompress_gzip(data: &[u8]) -> anyhow::Result<String> {
    let mut decoder = flate2::read::GzDecoder::new(data);
    let mut text = String::new();
    decoder.read_to_string(&mut text)?;
    Ok(text)
}

/// Parse DB-IP CSV format: ip_start,ip_end,country_code
fn parse_dbip_csv(csv: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();

    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 3 {
            continue;
        }

        let ip_start = parts[0];
        let ip_end = parts[1];
        let cc = parts[2];

        // Skip IPv6 (contains ':')
        if ip_start.contains(':') {
            continue;
        }

        if cc.len() != 2 {
            continue;
        }

        let start = match ip_to_u32(ip_start) {
            Some(ip) => ip,
            None => continue,
        };
        let end = match ip_to_u32(ip_end) {
            Some(ip) => ip,
            None => continue,
        };

        // Convert range to CIDR blocks
        for (net, prefix) in range_to_cidrs(start, end) {
            entries.push((format!("{}/{}", u32_to_ip(net), prefix), cc.to_string()));
        }
    }

    entries
}

/// Convert an IP range (start, end inclusive) to minimal set of CIDR blocks
fn range_to_cidrs(start: u32, end: u32) -> Vec<(u32, u8)> {
    let mut result = Vec::new();
    let mut current = start;

    while current <= end {
        // Max prefix bits based on alignment
        let trailing = if current == 0 {
            32
        } else {
            current.trailing_zeros()
        };
        let mut prefix_len = 32 - trailing as u8;

        // Shrink prefix until block fits within [current, end]
        loop {
            if prefix_len == 0 {
                break;
            }
            let block_end = current | ((1u64 << (32 - prefix_len as u64)) - 1) as u32;
            if block_end <= end {
                break;
            }
            prefix_len += 1;
        }

        result.push((current, prefix_len));

        let block_size = 1u64 << (32 - prefix_len as u64);
        let next = current as u64 + block_size;
        if next > u32::MAX as u64 {
            break;
        }
        current = next as u32;
    }

    result
}

fn ip_to_u32(ip: &str) -> Option<u32> {
    let parts: Vec<u8> = ip.split('.').filter_map(|p| p.parse().ok()).collect();
    if parts.len() != 4 {
        return None;
    }
    Some(
        (parts[0] as u32) << 24
            | (parts[1] as u32) << 16
            | (parts[2] as u32) << 8
            | parts[3] as u32,
    )
}

fn u32_to_ip(ip: u32) -> String {
    format!(
        "{}.{}.{}.{}",
        (ip >> 24) & 0xFF,
        (ip >> 16) & 0xFF,
        (ip >> 8) & 0xFF,
        ip & 0xFF,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_to_u32() {
        assert_eq!(ip_to_u32("192.168.1.1"), Some(0xC0A80101));
        assert_eq!(ip_to_u32("0.0.0.0"), Some(0));
        assert_eq!(ip_to_u32("255.255.255.255"), Some(0xFFFFFFFF));
        assert_eq!(ip_to_u32("invalid"), None);
    }

    #[test]
    fn test_u32_to_ip() {
        assert_eq!(u32_to_ip(0xC0A80101), "192.168.1.1");
        assert_eq!(u32_to_ip(0), "0.0.0.0");
    }

    #[test]
    fn test_range_to_cidrs_single_ip() {
        let result = range_to_cidrs(0xC0A80101, 0xC0A80101);
        assert_eq!(result, vec![(0xC0A80101, 32)]);
    }

    #[test]
    fn test_range_to_cidrs_aligned_block() {
        // 192.168.0.0 - 192.168.0.255 = /24
        let result = range_to_cidrs(0xC0A80000, 0xC0A800FF);
        assert_eq!(result, vec![(0xC0A80000, 24)]);
    }

    #[test]
    fn test_range_to_cidrs_unaligned() {
        // 1.0.0.0 - 1.0.0.2 = 3 IPs -> /32 + /31
        let result = range_to_cidrs(0x01000000, 0x01000002);
        let total: u64 = result.iter().map(|(_, p)| 1u64 << (32 - *p as u64)).sum();
        assert_eq!(total, 3);
    }

    #[test]
    fn test_parse_dbip_csv() {
        let csv = "1.0.0.0,1.0.0.255,AU\n1.0.1.0,1.0.1.255,CN\n::,::ffff,ZZ\n";
        let entries = parse_dbip_csv(csv);
        // Should have AU and CN entries, skip IPv6
        assert!(entries.iter().any(|(_, cc)| cc == "AU"));
        assert!(entries.iter().any(|(_, cc)| cc == "CN"));
        assert!(!entries.iter().any(|(_, cc)| cc == "ZZ"));
    }

    #[test]
    fn test_current_dbip_url() {
        let url = current_dbip_url();
        assert!(url.starts_with("https://download.db-ip.com/free/dbip-country-lite-"));
        assert!(url.ends_with(".csv.gz"));
    }
}
