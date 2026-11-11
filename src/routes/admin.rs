use askama::Template;
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;

use crate::{
    analytics::{
        exclude,
        reports::{self, FlowTransition},
        trend::{self, TrendChart},
    },
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
    #[serde(default)]
    page: Option<String>,
}

/// Pages other than posts that the dashboard can be narrowed to
const PAGES: [&str; 7] = [
    "/",
    "/blog",
    "/projects",
    "/about",
    "/resume",
    "/resume/print",
    "/contact",
];

fn default_period() -> String {
    "7d".into()
}

fn parse_period(s: &str) -> Option<i32> {
    s.strip_suffix('d').and_then(|n| n.parse().ok())
}

fn fmt_opt(v: Option<f64>) -> String {
    v.map(|f| format!("{:.1}", f)).unwrap_or_else(|| "-".into())
}

fn fmt_pct(n: i64, total: i64) -> String {
    if total > 0 {
        format!("{:.0}%", 100.0 * n as f64 / total as f64)
    } else {
        "-".into()
    }
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
    readers: i64,
    unconfirmed: i64,
    avg_scroll: String,
    max_scroll: String,
    median_time: String,
    long_reads: i64,
    conv_proj: String,
    conv_gh: String,
}

struct FunnelStepView {
    name: String,
    count: i64,
    pct: String,
}

struct PageView {
    path: String,
    views: i64,
    unique_sessions: i64,
}

struct CountryView {
    country: String,
    views: i64,
    pct: String,
}

struct ScopeOption {
    value: String,
    label: String,
    selected: bool,
}

/// What the dashboard is narrowed to: everything, one post or one other page
struct ScopeView {
    param: String,
    posts: Vec<ScopeOption>,
    pages: Vec<ScopeOption>,
    is_all: bool,
    is_post: bool,
}

struct SummaryView {
    page_views: i64,
    readers: i64,
    unconfirmed: i64,
    gone_fast: String,
    long_reads: i64,
    median_time: String,
}

struct TrendView {
    hourly: bool,
    chart: TrendChart,
}

struct SourceView {
    source: String,
    readers: i64,
    unconfirmed: i64,
    gone_fast: String,
    long_reads: i64,
    median_time: String,
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
    scope: ScopeView,
    summary: SummaryView,
    trend: TrendView,
    articles: Vec<ArticleView>,
    depth: Vec<FunnelStepView>,
    finished_fast: i64,
    sources: Vec<SourceView>,
    top_pages: Vec<PageView>,
    funnel: Vec<FunnelStepView>,
    countries: Vec<CountryView>,
    flow: Vec<FlowTransition>,
}

/// GET /admin/analytics?secret=...&period=7d&page=/blog/slug
pub async fn analytics_dashboard(
    State(state): State<AppState>,
    Query(query): Query<DashboardQuery>,
) -> Response {
    let Some(admin_secret) = &state.config.admin_secret else {
        return (
            StatusCode::FORBIDDEN,
            Html("Admin access not configured".to_string()),
        )
            .into_response();
    };
    if query.secret != *admin_secret {
        return (StatusCode::FORBIDDEN, Html("Invalid secret".to_string())).into_response();
    }

    let days = match parse_period(&query.period) {
        Some(d) => d,
        None => {
            return (StatusCode::BAD_REQUEST, Html("Invalid period".to_string())).into_response()
        }
    };

    let site_domain = state
        .config
        .site_url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    // Scope: all, one post or one other page; anything not in the lists falls back to all
    let posts: Vec<(String, String)> = state
        .content
        .read()
        .await
        .published_posts()
        .iter()
        .map(|p| {
            (
                format!("/blog/{}", p.metadata.slug),
                p.metadata.title.clone(),
            )
        })
        .collect();
    let path = query
        .page
        .as_deref()
        .filter(|p| posts.iter().any(|(path, _)| path == p) || PAGES.contains(p))
        .map(str::to_string);
    let path = path.as_deref();
    let is_post = path.is_some_and(|p| posts.iter().any(|(path, _)| path == p));
    let scope = ScopeView {
        param: path.unwrap_or_default().to_string(),
        posts: posts
            .iter()
            .map(|(p, title)| ScopeOption {
                value: p.clone(),
                label: title.clone(),
                selected: path == Some(p.as_str()),
            })
            .collect(),
        pages: PAGES
            .iter()
            .map(|p| ScopeOption {
                value: p.to_string(),
                label: p.to_string(),
                selected: path == Some(*p),
            })
            .collect(),
        is_all: path.is_none(),
        is_post,
    };

    let summary = match reports::summary_report(&state.pool, days, site_domain, path).await {
        Ok(s) => SummaryView {
            page_views: s.page_views,
            readers: s.readers,
            unconfirmed: s.unconfirmed,
            gone_fast: fmt_pct(s.gone_fast, s.visits),
            long_reads: s.long_reads,
            median_time: fmt_time(s.median_time),
        },
        Err(_) => SummaryView {
            page_views: 0,
            readers: 0,
            unconfirmed: 0,
            gone_fast: "-".into(),
            long_reads: 0,
            median_time: "-".into(),
        },
    };

    // Trend of the selected page, or of all posts summed; hourly for short periods
    let hourly = days <= 2;
    let chart = {
        let now = chrono::Utc::now();
        let starts = trend::bucket_starts(now, days, hourly);
        let rows = reports::page_trend(&state.pool, path, starts[0].and_utc(), hourly)
            .await
            .unwrap_or_default();
        let last_hour =
            reports::page_trend(&state.pool, path, now - chrono::Duration::hours(1), true)
                .await
                .unwrap_or_default()
                .iter()
                .fold((0, 0), |(r, h), row| (r + row.readers, h + row.halfway));
        let by_start: std::collections::HashMap<_, _> = rows
            .into_iter()
            .map(|r| (r.bucket, (r.readers, r.halfway)))
            .collect();
        let buckets: Vec<trend::Bucket> = starts
            .iter()
            .map(|start| {
                let (readers, halfway) = by_start.get(start).copied().unwrap_or((0, 0));
                trend::Bucket {
                    start: *start,
                    readers,
                    halfway,
                }
            })
            .collect();
        let elapsed = trend::elapsed_share(now, starts[starts.len() - 1], hourly);
        trend::build(&buckets, last_hour, elapsed, hourly)
    };
    let trend = TrendView { hourly, chart };

    let articles = if path.is_none() {
        reports::article_report(&state.pool, days, site_domain)
            .await
            .map(|r| {
                r.articles
                    .into_iter()
                    .map(|a| ArticleView {
                        path: a.path,
                        views: a.views,
                        readers: a.readers,
                        unconfirmed: a.unconfirmed,
                        avg_scroll: fmt_opt(a.avg_scroll),
                        max_scroll: fmt_opt(a.max_scroll),
                        median_time: fmt_time(a.median_time),
                        long_reads: a.long_reads,
                        conv_proj: fmt_opt(a.conv_projects),
                        conv_gh: fmt_opt(a.conv_github),
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let (depth, finished_fast) = match path {
        Some(p) => match reports::reading_depth(&state.pool, days, p).await {
            Ok(d) => {
                let step = |name: &str, count: i64| FunnelStepView {
                    name: name.into(),
                    count,
                    pct: if d.readers > 0 {
                        format!("{:.1}", 100.0 * count as f64 / d.readers as f64)
                    } else {
                        String::new()
                    },
                };
                (
                    vec![
                        step("Opened", d.readers),
                        step("Scrolled to 25%", d.scroll_25),
                        step("Scrolled to 50%", d.scroll_50),
                        step("Scrolled to 75%", d.scroll_75),
                        step("Reached the end", d.scroll_100),
                    ],
                    d.finished_fast,
                )
            }
            Err(_) => (Vec::new(), 0),
        },
        None => (Vec::new(), 0),
    };

    let sources = reports::source_report(&state.pool, days, site_domain, path)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|s| SourceView {
                    gone_fast: fmt_pct(s.gone_fast, s.readers),
                    source: s.source,
                    readers: s.readers,
                    unconfirmed: s.unconfirmed,
                    long_reads: s.long_reads,
                    median_time: fmt_time(s.median_time),
                })
                .collect()
        })
        .unwrap_or_default();

    let top_pages = if path.is_none() {
        reports::traffic_report(&state.pool, days, site_domain)
            .await
            .map(|t| {
                t.top_pages
                    .into_iter()
                    .map(|p| PageView {
                        path: p.path,
                        views: p.views,
                        unique_sessions: p.unique_sessions,
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let funnel_report = match path {
        None => Some(reports::funnel_report(&state.pool, days, site_domain).await),
        Some(p) if is_post => Some(reports::post_funnel(&state.pool, days, p).await),
        Some(_) => None,
    };
    let funnel = funnel_report
        .and_then(Result::ok)
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

    let raw_countries = reports::country_report(&state.pool, days, path)
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
        scope,
        summary,
        trend,
        articles,
        depth,
        finished_fast,
        sources,
        top_pages,
        funnel,
        countries,
        flow,
    };

    // Opening the dashboard marks this browser as the owner's: analytics stops tracking it
    (
        StatusCode::OK,
        [(
            header::SET_COOKIE,
            exclude::owner_cookie(state.config.is_production()),
        )],
        Html(
            template
                .render()
                .unwrap_or_else(|e| format!("Render error: {}", e)),
        ),
    )
        .into_response()
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
        "article" => match reports::article_report(&state.pool, days, sd).await {
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
        "country" => match reports::country_report(&state.pool, days, None).await {
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
