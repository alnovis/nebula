use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
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

/// Per-article metrics. Everything except `views` is counted per session, so reloads
/// do not weigh a session more than once.
#[derive(Serialize)]
pub struct ArticleStats {
    pub path: String,
    pub views: i64,
    pub readers: i64,
    pub unconfirmed: i64,
    pub avg_scroll: Option<f64>,
    pub max_scroll: Option<f64>,
    pub median_time: Option<f64>,
    pub long_reads: i64,
    pub conv_projects: Option<f64>,
    pub conv_github: Option<f64>,
}

/// A session counts as a long read after this much visible time on an article
pub const LONG_READ_SECS: i32 = 300;

/// Scroll depth (percent) at which a session has read an article halfway
pub const HALFWAY_DEPTH: f64 = 50.0;

/// A visit counts as gone fast below this much visible time
pub const GONE_FAST_SECS: i32 = 30;

/// Readers of one page, or of all posts, per hour or day (UTC), by the time of their first load
pub struct TrendRow {
    pub bucket: NaiveDateTime,
    pub readers: i64,
    pub halfway: i64,
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

/// Totals for the whole site or one path. Reading metrics are counted per visit, one per
/// (session, path), so on one path a visit is a reader.
#[derive(Serialize)]
pub struct SummaryStats {
    pub page_views: i64,
    pub readers: i64,
    pub unconfirmed: i64,
    pub visits: i64,
    pub gone_fast: i64,
    pub long_reads: i64,
    pub median_time: Option<f64>,
}

/// How far the readers of one path scrolled
#[derive(Serialize)]
pub struct ReadingDepth {
    pub readers: i64,
    pub scroll_25: i64,
    pub scroll_50: i64,
    pub scroll_75: i64,
    pub scroll_100: i64,
    pub finished_fast: i64,
}

/// Visits by referrer host. A visit is the first load of the path in a session, or the first
/// load of the session when no path is selected.
#[derive(Serialize)]
pub struct SourceStats {
    pub source: String,
    pub readers: i64,
    pub unconfirmed: i64,
    pub gone_fast: i64,
    pub long_reads: i64,
    pub median_time: Option<f64>,
}

/// SQL condition to filter known routes (excludes scanner/bot noise)
const KNOWN_ROUTES_FILTER: &str =
    "(pe.path = '/' OR pe.path = '/blog' OR pe.path LIKE '/blog/%' OR pe.path = '/projects' OR pe.path LIKE '/projects/%' OR pe.path = '/about' OR pe.path = '/resume' OR pe.path = '/resume/print' OR pe.path = '/contact' OR pe.path LIKE '/blog/tag/%')";

/// CTE prefix: human sessions = sessions with at least one client-side JS event
const HUMAN_SESSIONS_CTE: &str = "WITH hs AS (SELECT DISTINCT session_id FROM client_events)";

/// SQL condition over `s` (analytics_sessions): no client-side JS events, but a current browser
/// arriving from outside the site. Readers whose blocker drops the event beacon land here, and
/// so do some bots with a browser user agent, so counts over it are an upper bound. The version
/// floors are the last releases for old systems (Firefox 115 ESR, Chrome 109, Safari 15).
/// `$param` binds the internal referrer pattern (`%site_domain%`).
pub fn unconfirmed_session_cond(param: u8) -> String {
    format!(
        "NOT EXISTS (SELECT 1 FROM client_events ce WHERE ce.session_id = s.id)
         AND COALESCE(s.user_agent, '') ~ '^Mozilla/5\\.0 \\([^)]+\\) (AppleWebKit|Gecko)/'
         AND COALESCE(
               substring(s.user_agent from '(?:Firefox|FxiOS)/(\\d+)')::int >= 115,
               substring(s.user_agent from '(?:Chrome|CriOS)/(\\d+)')::int >= 109,
               substring(s.user_agent from 'Version/(\\d+)')::int >= 15,
               false)
         AND (s.first_referrer IS NULL OR s.first_referrer NOT LIKE ${param})"
    )
}

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

pub async fn article_report(
    pool: &PgPool,
    days: i32,
    site_domain: &str,
) -> anyhow::Result<ArticleReport> {
    let since = Utc::now() - chrono::Duration::days(days as i64);
    let internal_pattern = format!("%{}%", site_domain);

    // Single query over (article, session) pairs: page loads and readers, unconfirmed
    // sessions, avg/max scroll, median time, long reads, conversion to projects and to GitHub
    type Row = (
        String,
        i64,
        i64,
        i64,
        Option<f64>,
        Option<f64>,
        Option<f64>,
        i64,
        Option<f64>,
        Option<f64>,
    );
    let unconfirmed = unconfirmed_session_cond(3);
    let articles = sqlx::query_as::<_, Row>(&format!(
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
         conv_proj AS (
             SELECT DISTINCT session_id FROM page_events
             WHERE created_at >= $1 AND (path = '/projects' OR path LIKE '/projects/%')
         ),
         conv_gh AS (
             SELECT DISTINCT session_id FROM client_events
             WHERE created_at >= $1 AND event_type = 'outbound_click'
               AND event_data->>'url' LIKE '%github.com%'
         ),
         reads AS (
             SELECT pe.path, pe.session_id, COUNT(*) as loads
             FROM page_events pe JOIN hs ON hs.session_id = pe.session_id
             WHERE pe.created_at >= $1 AND pe.path LIKE '/blog/%'
             GROUP BY pe.path, pe.session_id
         ),
         unconf AS (
             SELECT pe.path, COUNT(DISTINCT pe.session_id) as sessions
             FROM page_events pe JOIN analytics_sessions s ON s.id = pe.session_id
             WHERE pe.created_at >= $1 AND pe.path LIKE '/blog/%' AND {unconfirmed}
             GROUP BY pe.path
         )
         SELECT r.path,
                SUM(r.loads)::bigint as views,
                COUNT(*) as readers,
                COALESCE(MAX(u.sessions), 0) as unconfirmed,
                AVG(sa.max_depth),
                MAX(sa.max_depth),
                PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY va.seconds),
                COUNT(*) FILTER (WHERE va.seconds >= $2),
                (100.0 * COUNT(cp.session_id) / COUNT(*))::float8,
                (100.0 * COUNT(cg.session_id) / COUNT(*))::float8
         FROM reads r
         LEFT JOIN scroll_agg sa ON sa.session_id = r.session_id AND sa.path = r.path
         LEFT JOIN vis_agg va ON va.session_id = r.session_id AND va.path = r.path
         LEFT JOIN conv_proj cp ON cp.session_id = r.session_id
         LEFT JOIN conv_gh cg ON cg.session_id = r.session_id
         LEFT JOIN unconf u ON u.path = r.path
         GROUP BY r.path ORDER BY readers DESC LIMIT 50"
    ))
    .bind(since)
    .bind(LONG_READ_SECS)
    .bind(&internal_pattern)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(
        |(
            path,
            views,
            readers,
            unconfirmed,
            avg_scroll,
            max_scroll,
            median_time,
            long_reads,
            conv_projects,
            conv_github,
        )| ArticleStats {
            path,
            views,
            readers,
            unconfirmed,
            avg_scroll,
            max_scroll,
            median_time,
            long_reads,
            conv_projects,
            conv_github,
        },
    )
    .collect();

    Ok(ArticleReport {
        period_days: days,
        articles,
    })
}

/// Trend of one path, or of all posts summed when `path` is `None`
pub async fn page_trend(
    pool: &PgPool,
    path: Option<&str>,
    since: DateTime<Utc>,
    hourly: bool,
) -> anyhow::Result<Vec<TrendRow>> {
    let rows = sqlx::query_as::<_, (NaiveDateTime, i64, i64)>(&format!(
        "{HUMAN_SESSIONS_CTE},
         first_load AS (
             SELECT pe.session_id, pe.path, MIN(pe.created_at) as at
             FROM page_events pe JOIN hs ON hs.session_id = pe.session_id
             WHERE pe.created_at >= $2
               AND (pe.path = $1::text
                    OR ($1::text IS NULL AND pe.path LIKE '/blog/%' AND pe.path NOT LIKE '/blog/tag/%'))
             GROUP BY pe.session_id, pe.path
         ),
         scroll_agg AS (
             SELECT session_id, path, MAX((event_data->>'depth')::float) as max_depth
             FROM client_events
             WHERE event_type = 'scroll' AND created_at >= $2
             GROUP BY session_id, path
         )
         SELECT date_trunc($3, fl.at AT TIME ZONE 'UTC') as bucket,
                COUNT(*),
                COUNT(*) FILTER (WHERE sa.max_depth >= $4)
         FROM first_load fl
         LEFT JOIN scroll_agg sa ON sa.session_id = fl.session_id AND sa.path = fl.path
         GROUP BY bucket ORDER BY bucket"
    ))
    .bind(path)
    .bind(since)
    .bind(if hourly { "hour" } else { "day" })
    .bind(HALFWAY_DEPTH)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(bucket, readers, halfway)| TrendRow {
        bucket,
        readers,
        halfway,
    })
    .collect();

    Ok(rows)
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
        "WITH hs AS (SELECT DISTINCT session_id FROM client_events)
         SELECT COUNT(DISTINCT pe.session_id)
         FROM page_events pe
         JOIN hs ON hs.session_id = pe.session_id
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
        "WITH hs AS (SELECT DISTINCT session_id FROM client_events)
         SELECT COUNT(DISTINCT pe.session_id)
         FROM page_events pe
         JOIN hs ON hs.session_id = pe.session_id
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

/// Countries of the whole site, or of one path
pub async fn country_report(
    pool: &PgPool,
    days: i32,
    path: Option<&str>,
) -> anyhow::Result<Vec<CountryStats>> {
    let since = Utc::now() - chrono::Duration::days(days as i64);

    let stats = sqlx::query_as::<_, (String, i64)>(&format!(
        "{HUMAN_SESSIONS_CTE}
         SELECT pe.country, COUNT(*) as views
         FROM page_events pe JOIN hs ON hs.session_id = pe.session_id
         WHERE pe.created_at >= $1 AND pe.country IS NOT NULL AND pe.country != 'ZZ' AND {KNOWN_ROUTES_FILTER}
           AND ($2::text IS NULL OR pe.path = $2)
         GROUP BY pe.country ORDER BY views DESC LIMIT 30"
    ))
    .bind(since)
    .bind(path)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(country, views)| CountryStats { country, views })
    .collect();

    Ok(stats)
}

/// Totals for the summary tiles: the whole site, or one path
pub async fn summary_report(
    pool: &PgPool,
    days: i32,
    site_domain: &str,
    path: Option<&str>,
) -> anyhow::Result<SummaryStats> {
    let since = Utc::now() - chrono::Duration::days(days as i64);
    let internal_pattern = format!("%{}%", site_domain);

    let (page_views, readers, visits, gone_fast, long_reads, median_time) =
        sqlx::query_as::<_, (i64, i64, i64, i64, i64, Option<f64>)>(&format!(
            "{HUMAN_SESSIONS_CTE},
             visits AS (
                 SELECT pe.session_id, pe.path, COUNT(*) as loads
                 FROM page_events pe JOIN hs ON hs.session_id = pe.session_id
                 WHERE pe.created_at >= $1 AND {KNOWN_ROUTES_FILTER}
                   AND ($2::text IS NULL OR pe.path = $2)
                 GROUP BY pe.session_id, pe.path
             ),
             vis_agg AS (
                 SELECT session_id, path, MAX((event_data->>'seconds')::float) as seconds
                 FROM client_events WHERE event_type = 'visibility' AND created_at >= $1
                 GROUP BY session_id, path
             )
             SELECT COALESCE(SUM(v.loads), 0)::bigint,
                    COUNT(DISTINCT v.session_id),
                    COUNT(*),
                    COUNT(*) FILTER (WHERE COALESCE(va.seconds, 0) < $3),
                    COUNT(*) FILTER (WHERE va.seconds >= $4),
                    PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY va.seconds)
             FROM visits v
             LEFT JOIN vis_agg va ON va.session_id = v.session_id AND va.path = v.path"
        ))
        .bind(since)
        .bind(path)
        .bind(GONE_FAST_SECS)
        .bind(LONG_READ_SECS)
        .fetch_one(pool)
        .await?;

    let unconfirmed = sqlx::query_as::<_, (i64,)>(&format!(
        "SELECT COUNT(DISTINCT pe.session_id)
         FROM page_events pe JOIN analytics_sessions s ON s.id = pe.session_id
         WHERE pe.created_at >= $1 AND {KNOWN_ROUTES_FILTER}
           AND ($2::text IS NULL OR pe.path = $2) AND {}",
        unconfirmed_session_cond(3)
    ))
    .bind(since)
    .bind(path)
    .bind(&internal_pattern)
    .fetch_one(pool)
    .await?
    .0;

    Ok(SummaryStats {
        page_views,
        readers,
        unconfirmed,
        visits,
        gone_fast,
        long_reads,
        median_time,
    })
}

/// Scroll depth reached by the readers of one path
pub async fn reading_depth(pool: &PgPool, days: i32, path: &str) -> anyhow::Result<ReadingDepth> {
    let since = Utc::now() - chrono::Duration::days(days as i64);

    let (readers, scroll_25, scroll_50, scroll_75, scroll_100, finished_fast) =
        sqlx::query_as::<_, (i64, i64, i64, i64, i64, i64)>(&format!(
            "{HUMAN_SESSIONS_CTE},
             readers AS (
                 SELECT DISTINCT pe.session_id
                 FROM page_events pe JOIN hs ON hs.session_id = pe.session_id
                 WHERE pe.created_at >= $1 AND pe.path = $2
             ),
             scroll_agg AS (
                 SELECT session_id, MAX((event_data->>'depth')::float) as max_depth
                 FROM client_events
                 WHERE event_type = 'scroll' AND path = $2 AND created_at >= $1
                 GROUP BY session_id
             ),
             vis_agg AS (
                 SELECT session_id, MAX((event_data->>'seconds')::float) as seconds
                 FROM client_events
                 WHERE event_type = 'visibility' AND path = $2 AND created_at >= $1
                 GROUP BY session_id
             )
             SELECT COUNT(*),
                    COUNT(*) FILTER (WHERE sa.max_depth >= 25),
                    COUNT(*) FILTER (WHERE sa.max_depth >= 50),
                    COUNT(*) FILTER (WHERE sa.max_depth >= 75),
                    COUNT(*) FILTER (WHERE sa.max_depth >= 100),
                    COUNT(*) FILTER (WHERE sa.max_depth >= 100 AND COALESCE(va.seconds, 0) < $3)
             FROM readers r
             LEFT JOIN scroll_agg sa ON sa.session_id = r.session_id
             LEFT JOIN vis_agg va ON va.session_id = r.session_id"
        ))
        .bind(since)
        .bind(path)
        .bind(LONG_READ_SECS)
        .fetch_one(pool)
        .await?;

    Ok(ReadingDepth {
        readers,
        scroll_25,
        scroll_50,
        scroll_75,
        scroll_100,
        finished_fast,
    })
}

/// Referrer hosts with the reading behaviour of the visits they brought
pub async fn source_report(
    pool: &PgPool,
    days: i32,
    site_domain: &str,
    path: Option<&str>,
) -> anyhow::Result<Vec<SourceStats>> {
    let since = Utc::now() - chrono::Duration::days(days as i64);
    let internal_pattern = format!("%{}%", site_domain);
    let unconfirmed = unconfirmed_session_cond(5);

    let sources = sqlx::query_as::<_, (String, i64, i64, i64, i64, Option<f64>)>(&format!(
        "{HUMAN_SESSIONS_CTE},
         entries AS (
             SELECT DISTINCT ON (pe.session_id) pe.session_id, pe.path, pe.referrer
             FROM page_events pe
             WHERE pe.created_at >= $1 AND {KNOWN_ROUTES_FILTER}
               AND ($2::text IS NULL OR pe.path = $2)
             ORDER BY pe.session_id, pe.created_at
         ),
         vis_agg AS (
             SELECT session_id, path, MAX((event_data->>'seconds')::float) as seconds
             FROM client_events WHERE event_type = 'visibility' AND created_at >= $1
             GROUP BY session_id, path
         )
         SELECT COALESCE(NULLIF(split_part(split_part(e.referrer, '://', 2), '/', 1), ''), '(direct)') as source,
                COUNT(*) FILTER (WHERE hs.session_id IS NOT NULL) as readers,
                COUNT(*) FILTER (WHERE hs.session_id IS NULL AND {unconfirmed}) as unconfirmed,
                COUNT(*) FILTER (WHERE hs.session_id IS NOT NULL AND COALESCE(va.seconds, 0) < $3),
                COUNT(*) FILTER (WHERE va.seconds >= $4),
                PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY va.seconds)
         FROM entries e
         LEFT JOIN analytics_sessions s ON s.id = e.session_id
         LEFT JOIN hs ON hs.session_id = e.session_id
         LEFT JOIN vis_agg va ON va.session_id = e.session_id AND va.path = e.path
         GROUP BY source
         HAVING COUNT(*) FILTER (WHERE hs.session_id IS NOT NULL) > 0
             OR COUNT(*) FILTER (WHERE hs.session_id IS NULL AND {unconfirmed}) > 0
         ORDER BY readers DESC, unconfirmed DESC LIMIT 20"
    ))
    .bind(since)
    .bind(path)
    .bind(GONE_FAST_SECS)
    .bind(LONG_READ_SECS)
    .bind(&internal_pattern)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(
        |(source, readers, unconfirmed, gone_fast, long_reads, median_time)| SourceStats {
            source,
            readers,
            unconfirmed,
            gone_fast,
            long_reads,
            median_time,
        },
    )
    .collect();

    Ok(sources)
}

/// Readers of one post and what they did after their first load of it
pub async fn post_funnel(pool: &PgPool, days: i32, path: &str) -> anyhow::Result<FunnelReport> {
    let since = Utc::now() - chrono::Duration::days(days as i64);

    let (readers, projects, github) = sqlx::query_as::<_, (i64, i64, i64)>(&format!(
        "{HUMAN_SESSIONS_CTE},
         first_read AS (
             SELECT pe.session_id, MIN(pe.created_at) as at
             FROM page_events pe JOIN hs ON hs.session_id = pe.session_id
             WHERE pe.created_at >= $1 AND pe.path = $2
             GROUP BY pe.session_id
         )
         SELECT COUNT(*),
                COUNT(*) FILTER (WHERE EXISTS (
                    SELECT 1 FROM page_events p
                    WHERE p.session_id = fr.session_id AND p.created_at > fr.at
                      AND (p.path = '/projects' OR p.path LIKE '/projects/%'))),
                COUNT(*) FILTER (WHERE EXISTS (
                    SELECT 1 FROM client_events ce
                    WHERE ce.session_id = fr.session_id AND ce.created_at > fr.at
                      AND ce.event_type = 'outbound_click'
                      AND ce.event_data->>'url' LIKE '%github.com%'))
         FROM first_read fr"
    ))
    .bind(since)
    .bind(path)
    .fetch_one(pool)
    .await?;

    let pct = |n: i64| -> Option<f64> {
        if readers > 0 {
            Some(100.0 * n as f64 / readers as f64)
        } else {
            None
        }
    };

    Ok(FunnelReport {
        period_days: days,
        steps: vec![
            FunnelStep {
                name: "Read the post".into(),
                count: readers,
                pct_of_first: pct(readers),
            },
            FunnelStep {
                name: "Visited projects after it".into(),
                count: projects,
                pct_of_first: pct(projects),
            },
            FunnelStep {
                name: "Clicked GitHub link after it".into(),
                count: github,
                pct_of_first: pct(github),
            },
        ],
    })
}
