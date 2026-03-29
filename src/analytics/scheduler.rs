use std::time::Duration;
use tokio::time;

use sqlx::PgPool;

use crate::state::AppState;

use super::reports;

/// Start the analytics report scheduler as a background task.
pub fn start(state: AppState) {
    let schedule = state
        .config
        .analytics_report_schedule
        .clone()
        .unwrap_or_default();
    let email = state.config.analytics_report_email.clone();

    if schedule.is_empty() || email.is_none() {
        tracing::info!("Analytics report scheduler disabled (ANALYTICS_REPORT_SCHEDULE or ANALYTICS_REPORT_EMAIL not set)");
        return;
    }

    let to_email = email.unwrap();
    let retention_days = state.config.analytics_retention_days;
    let (interval, period_days, label) = match schedule.as_str() {
        "daily" => (Duration::from_secs(86400), 1, "Daily"),
        _ => (Duration::from_secs(604800), 7, "Weekly"),
    };

    tracing::info!(
        "Analytics report scheduler started: {} reports to {}",
        label,
        to_email
    );

    let pool_for_cleanup = state.pool.clone();
    tokio::spawn(async move {
        // Wait before first report
        time::sleep(interval).await;

        loop {
            // Cleanup old data
            cleanup_old_data(&pool_for_cleanup, retention_days).await;

            tracing::info!("Generating scheduled {} analytics report", label);

            let report = match generate_email_report(&state, period_days).await {
                Ok(html) => html,
                Err(e) => {
                    tracing::error!("Failed to generate analytics report: {}", e);
                    time::sleep(interval).await;
                    continue;
                }
            };

            let subject = format!(
                "[Nebula] {} Analytics Report - {}",
                label,
                chrono::Utc::now().format("%Y-%m-%d")
            );

            if let Err(e) = state
                .email
                .send_html_email(&to_email, &subject, &report)
                .await
            {
                tracing::error!("Failed to send analytics report email: {}", e);
            }

            time::sleep(interval).await;
        }
    });
}

async fn generate_email_report(state: &AppState, days: i32) -> anyhow::Result<String> {
    let traffic = reports::traffic_report(&state.pool, days).await?;
    let site_domain = state
        .config
        .site_url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let funnel = reports::funnel_report(&state.pool, days, site_domain).await?;

    let mut html = String::from(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8">
<style>
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; color: #333; max-width: 600px; margin: 0 auto; padding: 20px; }
h1 { font-size: 20px; border-bottom: 2px solid #333; padding-bottom: 8px; }
h2 { font-size: 16px; color: #555; margin-top: 24px; }
table { width: 100%; border-collapse: collapse; margin: 12px 0; }
th, td { text-align: left; padding: 6px 10px; border-bottom: 1px solid #eee; }
th { font-weight: 600; background: #f5f5f5; }
.num { text-align: right; font-variant-numeric: tabular-nums; }
.stat { display: inline-block; text-align: center; margin: 0 16px 12px 0; }
.stat-value { font-size: 28px; font-weight: 700; }
.stat-label { font-size: 12px; color: #888; }
</style></head><body>"#,
    );

    html.push_str(&format!(
        "<h1>Analytics Report ({} days)</h1>",
        traffic.period_days
    ));

    // Summary stats
    html.push_str("<div>");
    html.push_str(&format!(
        r#"<div class="stat"><div class="stat-value">{}</div><div class="stat-label">Page Views</div></div>"#,
        traffic.total_page_views
    ));
    html.push_str(&format!(
        r#"<div class="stat"><div class="stat-value">{}</div><div class="stat-label">Unique Sessions</div></div>"#,
        traffic.unique_sessions
    ));
    html.push_str("</div>");

    // Top pages
    if !traffic.top_pages.is_empty() {
        html.push_str("<h2>Top Pages</h2><table><tr><th>Page</th><th class=\"num\">Views</th><th class=\"num\">Uniques</th></tr>");
        for p in traffic.top_pages.iter().take(10) {
            html.push_str(&format!(
                "<tr><td>{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td></tr>",
                p.path, p.views, p.unique_sessions
            ));
        }
        html.push_str("</table>");
    }

    // Top referrers
    if !traffic.top_referrers.is_empty() {
        html.push_str("<h2>Top Referrers</h2><table><tr><th>Referrer</th><th class=\"num\">Sessions</th></tr>");
        for r in traffic.top_referrers.iter().take(10) {
            html.push_str(&format!(
                "<tr><td>{}</td><td class=\"num\">{}</td></tr>",
                r.referrer, r.count
            ));
        }
        html.push_str("</table>");
    }

    // Funnel
    if !funnel.steps.is_empty() && funnel.steps[0].count > 0 {
        html.push_str("<h2>Referral Funnel</h2><table><tr><th>Step</th><th class=\"num\">Sessions</th><th class=\"num\">%</th></tr>");
        for s in &funnel.steps {
            html.push_str(&format!(
                "<tr><td>{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td></tr>",
                s.name,
                s.count,
                s.pct_of_first
                    .map(|p| format!("{:.1}%", p))
                    .unwrap_or_default()
            ));
        }
        html.push_str("</table>");
    }

    html.push_str("</body></html>");
    Ok(html)
}

/// Aggregate raw events into monthly summaries for data that's about to be deleted
async fn aggregate_monthly(
    pool: &PgPool,
    cutoff: chrono::DateTime<chrono::Utc>,
) -> Result<(), sqlx::Error> {
    // Aggregate page views by month, path, country
    let rows = sqlx::query(
        "INSERT INTO analytics_monthly (month, path, page_views, unique_sessions, avg_scroll_depth, avg_visibility_seconds, country)
         SELECT
           date_trunc('month', pe.created_at)::date AS month,
           pe.path,
           COUNT(*) AS page_views,
           COUNT(DISTINCT pe.session_id) AS unique_sessions,
           AVG((ce_scroll.event_data->>'depth')::real),
           AVG((ce_vis.event_data->>'seconds')::real),
           pe.country
         FROM page_events pe
         LEFT JOIN client_events ce_scroll
           ON ce_scroll.session_id = pe.session_id AND ce_scroll.path = pe.path AND ce_scroll.event_type = 'scroll'
         LEFT JOIN client_events ce_vis
           ON ce_vis.session_id = pe.session_id AND ce_vis.path = pe.path AND ce_vis.event_type = 'visibility'
         WHERE pe.created_at < $1
         GROUP BY month, pe.path, pe.country
         ON CONFLICT (month, path, country) DO UPDATE SET
           page_views = EXCLUDED.page_views,
           unique_sessions = EXCLUDED.unique_sessions,
           avg_scroll_depth = EXCLUDED.avg_scroll_depth,
           avg_visibility_seconds = EXCLUDED.avg_visibility_seconds",
    )
    .bind(cutoff)
    .execute(pool)
    .await?;

    if rows.rows_affected() > 0 {
        tracing::info!("Aggregated {} monthly page rows", rows.rows_affected());
    }

    // Aggregate referrers by month
    let rows = sqlx::query(
        "INSERT INTO analytics_monthly_referrers (month, referrer, sessions)
         SELECT
           date_trunc('month', started_at)::date AS month,
           first_referrer,
           COUNT(*)
         FROM analytics_sessions
         WHERE started_at < $1 AND first_referrer IS NOT NULL
         GROUP BY month, first_referrer
         ON CONFLICT (month, referrer) DO UPDATE SET
           sessions = EXCLUDED.sessions",
    )
    .bind(cutoff)
    .execute(pool)
    .await?;

    if rows.rows_affected() > 0 {
        tracing::info!("Aggregated {} monthly referrer rows", rows.rows_affected());
    }

    Ok(())
}

async fn cleanup_old_data(pool: &PgPool, retention_days: i32) {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(retention_days as i64);

    // Aggregate before deleting
    if let Err(e) = aggregate_monthly(pool, cutoff).await {
        tracing::error!("Monthly aggregation failed: {}", e);
    }

    let tables = ["page_events", "client_events"];
    for table in tables {
        match sqlx::query(&format!("DELETE FROM {} WHERE created_at < $1", table))
            .bind(cutoff)
            .execute(pool)
            .await
        {
            Ok(result) => {
                let count = result.rows_affected();
                if count > 0 {
                    tracing::info!(
                        "Cleanup: deleted {} rows from {} (older than {} days)",
                        count,
                        table,
                        retention_days
                    );
                }
            }
            Err(e) => tracing::error!("Cleanup failed for {}: {}", table, e),
        }
    }

    // Cleanup orphaned sessions (no events left)
    match sqlx::query(
        "DELETE FROM analytics_sessions WHERE started_at < $1
         AND id NOT IN (SELECT DISTINCT session_id FROM page_events)",
    )
    .bind(cutoff)
    .execute(pool)
    .await
    {
        Ok(result) => {
            let count = result.rows_affected();
            if count > 0 {
                tracing::info!("Cleanup: deleted {} orphaned sessions", count);
            }
        }
        Err(e) => tracing::error!("Cleanup failed for sessions: {}", e),
    }
}
