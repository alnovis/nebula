use std::time::Duration;
use tokio::time;

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
    let (interval, period_days, label) = match schedule.as_str() {
        "daily" => (Duration::from_secs(86400), 1, "Daily"),
        _ => (Duration::from_secs(604800), 7, "Weekly"),
    };

    tracing::info!(
        "Analytics report scheduler started: {} reports to {}",
        label,
        to_email
    );

    tokio::spawn(async move {
        // Wait before first report
        time::sleep(interval).await;

        loop {
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
    let funnel = reports::funnel_report(&state.pool, days).await?;

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
