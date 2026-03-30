use askama::Template;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use serde::Deserialize;

use crate::{
    analytics::reports::{self, FlowTransition},
    state::AppState,
    VERSION,
};

#[derive(Deserialize)]
pub struct ReloadQuery {
    secret: String,
}

#[derive(Deserialize)]
pub struct ReportQuery {
    secret: String,
    #[serde(rename = "type")]
    report_type: String,
    #[serde(default = "default_period")]
    period: String,
}

#[derive(Deserialize)]
pub struct DashboardQuery {
    secret: String,
    #[serde(default = "default_period")]
    period: String,
}

fn default_period() -> String {
    "7d".into()
}

fn parse_period(s: &str) -> Option<i32> {
    s.strip_suffix('d').and_then(|n| n.parse().ok())
}

fn fmt_opt(v: Option<f64>) -> String {
    v.map(|f| format!("{:.1}", f)).unwrap_or_else(|| "-".into())
}

fn fmt_time(v: Option<f64>) -> String {
    match v {
        Some(s) if s >= 60.0 => format!("{}m {:02}s", s as u64 / 60, s as u64 % 60),
        Some(s) => format!("{}s", s as u64),
        None => "-".into(),
    }
}

// -- View structs for template -----------------------------------------------

struct ArticleView {
    path: String,
    views: i64,
    avg_scroll: String,
    max_scroll: String,
    avg_time: String,
    conv_proj: String,
    conv_gh: String,
}

struct FunnelStepView {
    name: String,
    count: i64,
    pct: String,
}

struct SvgBar {
    x: String,
    y: String,
    w: String,
    h: String,
    date: String,
    count: i64,
}

struct SvgLine {
    y: String,
    label: String,
}

struct SvgLabel {
    x: String,
    text: String,
}

struct SvgCountLabel {
    x: String,
    y: String,
    text: String,
}

struct SvgChart {
    bars: Vec<SvgBar>,
    grid_lines: Vec<SvgLine>,
    x_labels: Vec<SvgLabel>,
    count_labels: Vec<SvgCountLabel>,
}

struct PageView {
    path: String,
    views: i64,
    unique_sessions: i64,
}

struct ReferrerView {
    referrer: String,
    count: i64,
}

struct CountryView {
    country: String,
    views: i64,
    pct: String,
}

struct TrafficView {
    total_page_views: i64,
    unique_sessions: i64,
    top_pages: Vec<PageView>,
    top_referrers: Vec<ReferrerView>,
    chart: SvgChart,
}

#[derive(Template)]
#[template(path = "admin/analytics.html")]
struct AnalyticsTemplate<'a> {
    title: &'a str,
    nav_path: &'a str,
    version: &'a str,
    canonical_url: String,
    og_type: &'a str,
    og_image: Option<String>,
    secret: &'a str,
    period_label: &'a str,
    traffic: TrafficView,
    articles: Vec<ArticleView>,
    funnel: Vec<FunnelStepView>,
    countries: Vec<CountryView>,
    flow: Vec<FlowTransition>,
}

/// GET /admin/analytics?secret=...&period=7d
pub async fn analytics_dashboard(
    State(state): State<AppState>,
    Query(query): Query<DashboardQuery>,
) -> impl IntoResponse {
    let Some(admin_secret) = &state.config.admin_secret else {
        return (
            StatusCode::FORBIDDEN,
            Html("Admin access not configured".to_string()),
        );
    };
    if query.secret != *admin_secret {
        return (StatusCode::FORBIDDEN, Html("Invalid secret".to_string()));
    }

    let days = match parse_period(&query.period) {
        Some(d) => d,
        None => return (StatusCode::BAD_REQUEST, Html("Invalid period".to_string())),
    };

    let site_domain = state
        .config
        .site_url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let raw_traffic = reports::traffic_report(&state.pool, days, site_domain)
        .await
        .ok();
    let traffic = {
        let t = raw_traffic.as_ref();
        TrafficView {
            total_page_views: t.map(|t| t.total_page_views).unwrap_or(0),
            unique_sessions: t.map(|t| t.unique_sessions).unwrap_or(0),
            top_pages: t
                .map(|t| {
                    t.top_pages
                        .iter()
                        .map(|p| PageView {
                            path: p.path.clone(),
                            views: p.views,
                            unique_sessions: p.unique_sessions,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            top_referrers: t
                .map(|t| {
                    t.top_referrers
                        .iter()
                        .map(|r| ReferrerView {
                            referrer: r.referrer.clone(),
                            count: r.count,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            chart: {
                // SVG chart constants
                const CL: f64 = 45.0; // chart left (after Y labels)
                const CR: f64 = 795.0; // chart right
                const CT: f64 = 15.0; // chart top
                const CB: f64 = 175.0; // chart bottom (X axis)
                const CW: f64 = CR - CL; // chart width
                const CH: f64 = CB - CT; // chart height

                let today = chrono::Utc::now().date_naive();
                let existing: std::collections::HashMap<chrono::NaiveDate, i64> = t
                    .map(|t| t.daily_views.iter().map(|d| (d.date, d.count)).collect())
                    .unwrap_or_default();
                let all_days: Vec<(chrono::NaiveDate, i64)> = (0..days)
                    .rev()
                    .map(|i| {
                        let date = today - chrono::Duration::days(i as i64);
                        let count = existing.get(&date).copied().unwrap_or(0);
                        (date, count)
                    })
                    .collect();
                let n = all_days.len();
                let max_val = all_days.iter().map(|(_, c)| *c).max().unwrap_or(1).max(1);

                // Bars
                let slot = CW / n as f64;
                let bar_w = (slot * 0.7).clamp(2.0, 40.0);
                let bars: Vec<SvgBar> = all_days
                    .iter()
                    .enumerate()
                    .map(|(i, (d, c))| {
                        let cx = CL + (i as f64 + 0.5) * slot;
                        let h = if max_val > 0 {
                            (*c as f64 / max_val as f64) * CH
                        } else {
                            0.0
                        };
                        SvgBar {
                            x: format!("{:.1}", cx - bar_w / 2.0),
                            y: format!("{:.1}", CB - h),
                            w: format!("{:.1}", bar_w),
                            h: format!("{:.1}", h),
                            date: d.to_string(),
                            count: *c,
                        }
                    })
                    .collect();

                // Y grid lines
                let raw_step = max_val as f64 / 4.0;
                let magnitude = 10f64.powf(raw_step.log10().floor());
                let step = if raw_step / magnitude < 1.5 {
                    magnitude as i64
                } else if raw_step / magnitude < 3.5 {
                    (2.0 * magnitude) as i64
                } else {
                    (5.0 * magnitude) as i64
                }
                .max(1);
                let mut grid_lines = Vec::new();
                let mut v = step;
                while v <= max_val {
                    let y = CB - (v as f64 / max_val as f64) * CH;
                    grid_lines.push(SvgLine {
                        y: format!("{:.1}", y),
                        label: v.to_string(),
                    });
                    v += step;
                }

                // X labels
                let date_fmt = if days <= 14 { "%b %d" } else { "%m/%d" };
                let target = if days <= 7 { days as usize } else { 6 }.min(n);
                let x_labels: Vec<SvgLabel> = (0..target)
                    .map(|k| {
                        let idx = if target <= 1 {
                            0
                        } else {
                            k * (n - 1) / (target - 1)
                        };
                        let cx = CL + (idx as f64 + 0.5) * slot;
                        SvgLabel {
                            x: format!("{:.1}", cx),
                            text: all_days[idx].0.format(date_fmt).to_string(),
                        }
                    })
                    .collect();

                // Count labels (max value + last day)
                let last_idx = n.saturating_sub(1);
                let max_idx = all_days
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, (_, c))| *c)
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                let mut count_labels = Vec::new();
                for idx in [max_idx, last_idx] {
                    if all_days[idx].1 > 0
                        && !count_labels.iter().any(|l: &SvgCountLabel| {
                            l.x == format!("{:.1}", CL + (idx as f64 + 0.5) * slot)
                        })
                    {
                        let cx = CL + (idx as f64 + 0.5) * slot;
                        let h = (all_days[idx].1 as f64 / max_val as f64) * CH;
                        count_labels.push(SvgCountLabel {
                            x: format!("{:.1}", cx),
                            y: format!("{:.1}", CB - h - 5.0),
                            text: all_days[idx].1.to_string(),
                        });
                    }
                }

                SvgChart {
                    bars,
                    grid_lines,
                    x_labels,
                    count_labels,
                }
            },
        }
    };

    let articles = reports::article_report(&state.pool, days)
        .await
        .map(|r| {
            r.articles
                .into_iter()
                .map(|a| ArticleView {
                    path: a.path,
                    views: a.views,
                    avg_scroll: fmt_opt(a.avg_scroll),
                    max_scroll: fmt_opt(a.max_scroll),
                    avg_time: fmt_time(a.avg_time),
                    conv_proj: fmt_opt(a.conv_projects),
                    conv_gh: fmt_opt(a.conv_github),
                })
                .collect()
        })
        .unwrap_or_default();

    let funnel = reports::funnel_report(&state.pool, days, site_domain)
        .await
        .map(|r| {
            r.steps
                .into_iter()
                .map(|s| FunnelStepView {
                    name: s.name,
                    count: s.count,
                    pct: s
                        .pct_of_first
                        .map(|p| format!("{:.1}", p))
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default();

    let raw_countries = reports::country_report(&state.pool, days)
        .await
        .unwrap_or_default();
    let country_total: i64 = raw_countries.iter().map(|c| c.views).sum::<i64>().max(1);
    let countries: Vec<CountryView> = raw_countries
        .into_iter()
        .map(|c| {
            let label = if c.country == "ZZ" {
                "Unknown".to_string()
            } else {
                c.country
            };
            CountryView {
                pct: format!("{:.1}", 100.0 * c.views as f64 / country_total as f64),
                country: label,
                views: c.views,
            }
        })
        .collect();

    let flow = reports::flow_report(&state.pool, days)
        .await
        .map(|r| r.transitions)
        .unwrap_or_default();

    let template = AnalyticsTemplate {
        title: "Analytics",
        nav_path: "/admin",
        version: VERSION,
        canonical_url: format!("{}/admin/analytics", state.config.site_url),
        og_type: "website",
        og_image: None,
        secret: &query.secret,
        period_label: &query.period,
        traffic,
        articles,
        funnel,
        countries,
        flow,
    };

    (
        StatusCode::OK,
        Html(
            template
                .render()
                .unwrap_or_else(|e| format!("Render error: {}", e)),
        ),
    )
}

/// GET /admin/analytics/report?secret=...&type=traffic&period=7d
/// Returns JSON for programmatic access.
pub async fn analytics_report(
    State(state): State<AppState>,
    Query(query): Query<ReportQuery>,
) -> impl IntoResponse {
    let Some(admin_secret) = &state.config.admin_secret else {
        return (
            StatusCode::FORBIDDEN,
            "Admin access not configured".to_string(),
        );
    };
    if query.secret != *admin_secret {
        return (StatusCode::FORBIDDEN, "Invalid secret".to_string());
    }

    let days = match parse_period(&query.period) {
        Some(d) => d,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                "Invalid period format (use 7d, 30d, etc.)".to_string(),
            )
        }
    };

    let sd = state
        .config
        .site_url
        .trim_start_matches("https://")
        .trim_start_matches("http://");

    match query.report_type.as_str() {
        "traffic" => match reports::traffic_report(&state.pool, days, sd).await {
            Ok(r) => (
                StatusCode::OK,
                serde_json::to_string_pretty(&r).unwrap_or_default(),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Report error: {}", e),
            ),
        },
        "article" => match reports::article_report(&state.pool, days).await {
            Ok(r) => (
                StatusCode::OK,
                serde_json::to_string_pretty(&r).unwrap_or_default(),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Report error: {}", e),
            ),
        },
        "funnel" => match reports::funnel_report(&state.pool, days, sd).await {
            Ok(r) => (
                StatusCode::OK,
                serde_json::to_string_pretty(&r).unwrap_or_default(),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Report error: {}", e),
            ),
        },
        "flow" => match reports::flow_report(&state.pool, days).await {
            Ok(r) => (
                StatusCode::OK,
                serde_json::to_string_pretty(&r).unwrap_or_default(),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Report error: {}", e),
            ),
        },
        "country" => match reports::country_report(&state.pool, days).await {
            Ok(r) => (
                StatusCode::OK,
                serde_json::to_string_pretty(&r).unwrap_or_default(),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Report error: {}", e),
            ),
        },
        _ => (
            StatusCode::BAD_REQUEST,
            "Unknown report type (traffic, article, funnel, flow, country)".to_string(),
        ),
    }
}

/// Reload content from filesystem
/// Usage: POST /admin/reload?secret=YOUR_SECRET
pub async fn reload_content(
    State(state): State<AppState>,
    Query(query): Query<ReloadQuery>,
) -> impl IntoResponse {
    // Check if admin secret is configured
    let Some(admin_secret) = &state.config.admin_secret else {
        return (StatusCode::FORBIDDEN, "Admin access not configured");
    };

    // Validate secret
    if query.secret != *admin_secret {
        return (StatusCode::FORBIDDEN, "Invalid secret");
    }

    // Reload content
    match state.reload_content().await {
        Ok(_) => {
            tracing::info!("Content reloaded successfully");
            (StatusCode::OK, "Content reloaded successfully")
        }
        Err(e) => {
            tracing::error!("Failed to reload content: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to reload content",
            )
        }
    }
}
