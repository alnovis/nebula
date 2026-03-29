use sqlx::PgPool;
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
pub fn start_updater(pool: PgPool, client: reqwest::Client, rir_urls: Vec<String>) {
    if rir_urls.is_empty() {
        tracing::info!("GeoIP updater disabled (no RIR URLs configured)");
        return;
    }

    tokio::spawn(async move {
        // Initial load on startup (after short delay)
        time::sleep(Duration::from_secs(10)).await;
        if let Err(e) = update_geoip_data(&pool, &client, &rir_urls).await {
            tracing::error!("Initial GeoIP update failed: {}", e);
        }

        // Then run every 24 hours
        let mut interval = time::interval(Duration::from_secs(86400));
        interval.tick().await; // skip first immediate tick
        loop {
            interval.tick().await;
            tracing::info!("Running nightly GeoIP update");
            if let Err(e) = update_geoip_data(&pool, &client, &rir_urls).await {
                tracing::error!("GeoIP update failed: {}", e);
            }
        }
    });
}

/// Download all RIR delegation files and refresh geoip_ranges table
async fn update_geoip_data(
    pool: &PgPool,
    client: &reqwest::Client,
    rir_urls: &[String],
) -> anyhow::Result<()> {
    let mut entries: Vec<(String, String)> = Vec::new(); // (cidr, country)

    for url in rir_urls {
        tracing::info!("Downloading RIR data: {}", url);
        match download_and_parse(client, url).await {
            Ok(mut parsed) => {
                tracing::info!("  Parsed {} IPv4 entries", parsed.len());
                entries.append(&mut parsed);
            }
            Err(e) => {
                tracing::warn!("Failed to download {}: {}", url, e);
            }
        }
    }

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

    // Batch insert using unnest for performance
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

/// Download and parse a single RIR delegation file
async fn download_and_parse(
    client: &reqwest::Client,
    url: &str,
) -> anyhow::Result<Vec<(String, String)>> {
    let response = client.get(url).send().await?;
    let text = response.text().await?;
    let mut entries = Vec::new();

    for line in text.lines() {
        // Skip comments and headers
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        // Format: registry|cc|type|start|value|date|status[|extensions]
        if parts.len() < 7 {
            continue;
        }

        let cc = parts[1];
        let ip_type = parts[2];
        let start = parts[3];
        let value = parts[4];

        // Only process IPv4, skip summary lines and wildcards
        if ip_type != "ipv4" || cc == "*" || cc.len() != 2 {
            continue;
        }

        let count: u32 = match value.parse() {
            Ok(n) => n,
            Err(_) => continue,
        };

        let start_ip: u32 = match ip_to_u32(start) {
            Some(ip) => ip,
            None => continue,
        };

        // Convert count to CIDR prefix length
        if let Some(prefix_len) = count_to_prefix(count) {
            let normalized = normalize_cidr(start_ip, prefix_len);
            entries.push((
                format!("{}/{}", u32_to_ip(normalized), prefix_len),
                cc.to_string(),
            ));
        } else {
            // Non-power-of-2 allocation: split into multiple CIDR blocks
            for (sub_start, sub_prefix) in split_to_cidrs(start_ip, count) {
                let normalized = normalize_cidr(sub_start, sub_prefix);
                entries.push((
                    format!("{}/{}", u32_to_ip(normalized), sub_prefix),
                    cc.to_string(),
                ));
            }
        }
    }

    Ok(entries)
}

/// Zero out host bits to make a valid CIDR network address
fn normalize_cidr(ip: u32, prefix_len: u8) -> u32 {
    if prefix_len >= 32 {
        ip
    } else {
        let mask = !((1u32 << (32 - prefix_len)) - 1);
        ip & mask
    }
}

/// Convert IP count to CIDR prefix length (only works for powers of 2)
fn count_to_prefix(count: u32) -> Option<u8> {
    if count == 0 || (count & (count - 1)) != 0 {
        return None; // not a power of 2
    }
    Some(32 - count.trailing_zeros() as u8)
}

/// Parse dotted-decimal IPv4 to u32
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

/// Convert u32 to dotted-decimal IPv4
fn u32_to_ip(ip: u32) -> String {
    format!(
        "{}.{}.{}.{}",
        (ip >> 24) & 0xFF,
        (ip >> 16) & 0xFF,
        (ip >> 8) & 0xFF,
        ip & 0xFF,
    )
}

/// Split a non-power-of-2 range into valid CIDR blocks
fn split_to_cidrs(start: u32, count: u32) -> Vec<(u32, u8)> {
    let mut result = Vec::new();
    let mut remaining = count;
    let mut current = start;

    while remaining > 0 {
        // Largest power-of-2 block that fits and is aligned
        let max_bits = remaining.next_power_of_two().trailing_zeros();
        let alignment = if current == 0 {
            32
        } else {
            current.trailing_zeros()
        };
        let bits = max_bits.min(alignment);
        let block_size = 1u32 << bits;

        // Don't overshoot
        let bits = if block_size > remaining {
            (remaining as f64).log2().floor() as u32
        } else {
            bits
        };
        let block_size = 1u32 << bits;

        result.push((current, 32 - bits as u8));
        current += block_size;
        remaining -= block_size;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_to_prefix() {
        assert_eq!(count_to_prefix(256), Some(24));
        assert_eq!(count_to_prefix(65536), Some(16));
        assert_eq!(count_to_prefix(1), Some(32));
        assert_eq!(count_to_prefix(1024), Some(22));
        assert_eq!(count_to_prefix(0), None);
        assert_eq!(count_to_prefix(100), None); // not power of 2
    }

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
    fn test_split_to_cidrs_power_of_2() {
        let result = split_to_cidrs(0xC0A80000, 256); // 192.168.0.0/24
        assert_eq!(result, vec![(0xC0A80000, 24)]);
    }

    #[test]
    fn test_split_to_cidrs_non_power_of_2() {
        let result = split_to_cidrs(0x0A000000, 768); // 10.0.0.0, 768 IPs
                                                      // 768 = 512 + 256 -> /23 + /24
        assert_eq!(result.len(), 2);
        let total: u32 = result.iter().map(|(_, p)| 1u32 << (32 - *p as u32)).sum();
        assert_eq!(total, 768);
    }
}
