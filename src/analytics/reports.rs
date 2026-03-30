use chrono::{NaiveDate, Utc};
use serde::Serialize;
use sqlx::PgPool;

// -- Report types ------------------------------------------------------------

#[derive(Serialize)]
pub struct TrafficReport {
    pub period_days: i32,
    pub total_page_views: i64,
    pub unique_sessions: i64,
    pub top_pages: Vec<PageStats>,
    pub top_referrers: Vec<ReferrerStats>,
    pub daily_views: Vec<DailyCount>,
}

#[derive(Serialize)]
pub struct PageStats {
    pub path: String,
    pub views: i64,
    pub unique_sessions: i64,
}

#[derive(Serialize)]
pub struct ReferrerStats {
    pub referrer: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct DailyCount {
    pub date: NaiveDate,
    pub count: i64,
}

#[derive(Serialize)]
pub struct ArticleReport {
    pub period_days: i32,
    pub articles: Vec<ArticleStats>,
}

#[derive(Serialize)]
pub struct ArticleStats {
    pub path: String,
    pub views: i64,
    pub avg_scroll_depth: Option<f64>,
    pub avg_visibility_seconds: Option<f64>,
    pub bounce_rate: Option<f64>,
}

#[derive(Serialize)]
pub struct FunnelReport {
    pub period_days: i32,
    pub steps: Vec<FunnelStep>,
}

#[derive(Serialize)]
pub struct FunnelStep {
    pub name: String,
    pub count: i64,
    pub pct_of_first: Option<f64>,
}

#[derive(Serialize)]
pub struct FlowReport {
    pub period_days: i32,
    pub transitions: Vec<FlowTransition>,
}

#[derive(Serialize)]
pub struct CountryStats {
    pub country: String,
    pub views: i64,
}

#[derive(Serialize)]
pub struct FlowTransition {
    pub from_path: String,
    pub to_path: String,
    pub count: i64,
}

/// SQL condition to filter known routes (excludes scanner/bot noise)
const KNOWN_ROUTES_FILTER: &str =
    "(pe.path = '/' OR pe.path = '/blog' OR pe.path LIKE '/blog/%' OR pe.path = '/projects' OR pe.path LIKE '/projects/%' OR pe.path = '/about' OR pe.path = '/resume' OR pe.path = '/resume/print' OR pe.path = '/contact' OR pe.path LIKE '/blog/tag/%')";

/// CTE prefix: human sessions = sessions with at least one client-side JS event
const HUMAN_SESSIONS_CTE: &str = "WITH hs AS (SELECT DISTINCT session_id FROM client_events)";

// -- Report generation -------------------------------------------------------

pub async fn traffic_report(
    pool: &PgPool,
    days: i32,
    site_domain: &str,
) -> anyhow::Result<TrafficReport> {
    let since = Utc::now() - chrono::Duration::days(days as i64);
    let internal_pattern = format!("%{}%", site_domain);

    let totals = sqlx::query_as::<_, (i64, i64)>(&format!(
        "{HUMAN_SESSIONS_CTE}
         SELECT COUNT(*), COUNT(DISTINCT pe.session_id)
         FROM page_events pe JOIN hs ON hs.session_id = pe.session_id
         WHERE pe.created_at >= $1 AND {KNOWN_ROUTES_FILTER}"
    ))
    .bind(since)
    .fetch_one(pool)
    .await?;

    let top_pages = sqlx::query_as::<_, (String, i64, i64)>(&format!(
        "{HUMAN_SESSIONS_CTE}
         SELECT pe.path, COUNT(*) as views, COUNT(DISTINCT pe.session_id) as uniq
         FROM page_events pe JOIN hs ON hs.session_id = pe.session_id
         WHERE pe.created_at >= $1 AND {KNOWN_ROUTES_FILTER}
         GROUP BY pe.path ORDER BY views DESC LIMIT 20"
    ))
    .bind(since)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(path, views, unique_sessions)| PageStats {
        path,
        views,
        unique_sessions,
    })
    .collect();

    let top_referrers = sqlx::query_as::<_, (String, i64)>(
        "WITH hs AS (SELECT DISTINCT session_id FROM client_events)
         SELECT s.first_referrer, COUNT(*) as cnt
         FROM analytics_sessions s JOIN hs ON hs.session_id = s.id
         WHERE s.started_at >= $1 AND s.first_referrer IS NOT NULL
           AND s.first_referrer NOT LIKE $2
         GROUP BY s.first_referrer ORDER BY cnt DESC LIMIT 20",
    )
    .bind(since)
    .bind(&internal_pattern)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(referrer, count)| ReferrerStats { referrer, count })
    .collect();

    let daily_views = sqlx::query_as::<_, (NaiveDate, i64)>(&format!(
        "{HUMAN_SESSIONS_CTE}
         SELECT pe.created_at::date as day, COUNT(*)
         FROM page_events pe JOIN hs ON hs.session_id = pe.session_id
         WHERE pe.created_at >= $1 AND {KNOWN_ROUTES_FILTER}
         GROUP BY day ORDER BY day"
    ))
    .bind(since)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(date, count)| DailyCount { date, count })
    .collect();

    Ok(TrafficReport {
        period_days: days,
        total_page_views: totals.0,
        unique_sessions: totals.1,
        top_pages,
        top_referrers,
        daily_views,
    })
}

pub async fn article_report(pool: &PgPool, days: i32) -> anyhow::Result<ArticleReport> {
    let since = Utc::now() - chrono::Duration::days(days as i64);

    // Single query: views, scroll depth (MAX per session → AVG), visibility, bounce rate
    let articles = sqlx::query_as::<_, (String, i64, Option<f64>, Option<f64>, Option<f64>)>(
        "WITH hs AS (SELECT DISTINCT session_id FROM client_events),
         scroll_agg AS (
             SELECT session_id, path, MAX((event_data->>'depth')::float) as max_depth
             FROM client_events WHERE event_type = 'scroll' AND created_at >= $1
             GROUP BY session_id, path
         ),
         vis_agg AS (
             SELECT session_id, path, MAX((event_data->>'seconds')::float) as seconds
             FROM client_events WHERE event_type = 'visibility' AND created_at >= $1
             GROUP BY session_id, path
         ),
         session_counts AS (
             SELECT session_id, COUNT(*) as cnt
             FROM page_events WHERE created_at >= $1
             GROUP BY session_id
         )
         SELECT pe.path,
                COUNT(*) as views,
                AVG(sa.max_depth),
                AVG(va.seconds),
                CASE WHEN COUNT(DISTINCT pe.session_id) = 0 THEN NULL
                     ELSE 100.0 * COUNT(DISTINCT pe.session_id)
                          FILTER (WHERE sc.cnt = 1) / COUNT(DISTINCT pe.session_id)::float
                END as bounce
         FROM page_events pe
         JOIN hs ON hs.session_id = pe.session_id
         LEFT JOIN scroll_agg sa ON sa.session_id = pe.session_id AND sa.path = pe.path
         LEFT JOIN vis_agg va ON va.session_id = pe.session_id AND va.path = pe.path
         LEFT JOIN session_counts sc ON sc.session_id = pe.session_id
         WHERE pe.created_at >= $1 AND pe.path LIKE '/blog/%'
         GROUP BY pe.path ORDER BY views DESC LIMIT 50",
    )
    .bind(since)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(
        |(path, views, avg_scroll_depth, avg_visibility_seconds, bounce_rate)| ArticleStats {
            path,
            views,
            avg_scroll_depth,
            avg_visibility_seconds,
            bounce_rate,
        },
    )
    .collect();

    Ok(ArticleReport {
        period_days: days,
        articles,
    })
}

pub async fn funnel_report(
    pool: &PgPool,
    days: i32,
    site_domain: &str,
) -> anyhow::Result<FunnelReport> {
    let since = Utc::now() - chrono::Duration::days(days as i64);
    let internal_pattern = format!("%{}%", site_domain);

    // Step 1: Sessions with external referrer (exclude own domain), verified human only
    let total_sessions = sqlx::query_as::<_, (i64,)>(
        "WITH hs AS (SELECT DISTINCT session_id FROM client_events)
         SELECT COUNT(*) FROM analytics_sessions s
         JOIN hs ON hs.session_id = s.id
         WHERE s.started_at >= $1 AND s.first_referrer IS NOT NULL
           AND s.first_referrer NOT LIKE $2",
    )
    .bind(since)
    .bind(&internal_pattern)
    .fetch_one(pool)
    .await?
    .0;

    // Step 2: Sessions that visited a blog post
    let blog_sessions = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(DISTINCT pe.session_id)
         FROM page_events pe
         JOIN analytics_sessions s ON pe.session_id = s.id
         WHERE s.started_at >= $1 AND s.first_referrer IS NOT NULL
           AND s.first_referrer NOT LIKE $2
           AND pe.path LIKE '/blog/%'",
    )
    .bind(since)
    .bind(&internal_pattern)
    .fetch_one(pool)
    .await?
    .0;

    // Step 3: Sessions that visited projects
    let project_sessions = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(DISTINCT pe.session_id)
         FROM page_events pe
         JOIN analytics_sessions s ON pe.session_id = s.id
         WHERE s.started_at >= $1 AND s.first_referrer IS NOT NULL
           AND s.first_referrer NOT LIKE $2
           AND (pe.path = '/projects' OR pe.path LIKE '/projects/%')",
    )
    .bind(since)
    .bind(&internal_pattern)
    .fetch_one(pool)
    .await?
    .0;

    // Step 4: Sessions that clicked a GitHub link
    let github_clicks = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(DISTINCT ce.session_id)
         FROM client_events ce
         JOIN analytics_sessions s ON ce.session_id = s.id
         WHERE s.started_at >= $1 AND s.first_referrer IS NOT NULL
           AND s.first_referrer NOT LIKE $2
           AND ce.event_type = 'outbound_click'
           AND ce.event_data->>'url' LIKE '%github.com%'",
    )
    .bind(since)
    .bind(&internal_pattern)
    .fetch_one(pool)
    .await?
    .0;

    let pct = |n: i64| -> Option<f64> {
        if total_sessions > 0 {
            Some(100.0 * n as f64 / total_sessions as f64)
        } else {
            None
        }
    };

    Ok(FunnelReport {
        period_days: days,
        steps: vec![
            FunnelStep {
                name: "External referral sessions".into(),
                count: total_sessions,
                pct_of_first: Some(100.0),
            },
            FunnelStep {
                name: "Visited blog post".into(),
                count: blog_sessions,
                pct_of_first: pct(blog_sessions),
            },
            FunnelStep {
                name: "Visited projects".into(),
                count: project_sessions,
                pct_of_first: pct(project_sessions),
            },
            FunnelStep {
                name: "Clicked GitHub link".into(),
                count: github_clicks,
                pct_of_first: pct(github_clicks),
            },
        ],
    })
}

pub async fn flow_report(pool: &PgPool, days: i32) -> anyhow::Result<FlowReport> {
    let since = Utc::now() - chrono::Duration::days(days as i64);

    let transitions = sqlx::query_as::<_, (String, String, i64)>(&format!(
        "{HUMAN_SESSIONS_CTE}
         SELECT pe.prev_path, pe.path, COUNT(*) as cnt
         FROM page_events pe JOIN hs ON hs.session_id = pe.session_id
         WHERE pe.prev_path IS NOT NULL AND pe.created_at >= $1
           AND {KNOWN_ROUTES_FILTER}
         GROUP BY pe.prev_path, pe.path
         ORDER BY cnt DESC LIMIT 50"
    ))
    .bind(since)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(from_path, to_path, count)| FlowTransition {
        from_path,
        to_path,
        count,
    })
    .collect();

    Ok(FlowReport {
        period_days: days,
        transitions,
    })
}

pub async fn country_report(pool: &PgPool, days: i32) -> anyhow::Result<Vec<CountryStats>> {
    let since = Utc::now() - chrono::Duration::days(days as i64);

    let stats = sqlx::query_as::<_, (String, i64)>(&format!(
        "{HUMAN_SESSIONS_CTE}
         SELECT pe.country, COUNT(*) as views
         FROM page_events pe JOIN hs ON hs.session_id = pe.session_id
         WHERE pe.created_at >= $1 AND pe.country IS NOT NULL AND pe.country != 'ZZ' AND {KNOWN_ROUTES_FILTER}
         GROUP BY pe.country ORDER BY views DESC LIMIT 30"
    ))
    .bind(since)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(country, views)| CountryStats { country, views })
    .collect();

    Ok(stats)
}
