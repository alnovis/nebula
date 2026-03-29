# Analytics

Privacy-first behavioral analytics: server-side middleware + lightweight client JS. No third-party scripts, no PII stored.

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `ADMIN_SECRET` | -- | Secret token for `/admin/*` endpoints (required for analytics access) |
| `REDIS_URL` | -- | Redis connection for session state (optional, enables prev_path tracking) |
| `ANALYTICS_REPORT_SCHEDULE` | disabled | `daily` or `weekly` -- enables scheduled email reports |
| `ANALYTICS_REPORT_EMAIL` | -- | Recipient for scheduled reports |
| `ANALYTICS_RETENTION_DAYS` | `90` | Days to keep raw event data before aggregation + cleanup |
| `GEOIP_URL` | auto | Custom DB-IP CSV URL (auto-generated from current month if not set) |

## Endpoints

### Dashboard

```
GET /admin/analytics?secret=SECRET&period=7d
```

HTML dashboard with daily views chart, top pages, referrers, article performance, referral funnel, country breakdown, navigation flow.

| Param | Required | Default | Values |
|-------|----------|---------|--------|
| `secret` | yes | -- | `ADMIN_SECRET` value |
| `period` | no | `7d` | `7d`, `14d`, `30d`, `90d` |

### JSON Reports

```
GET /admin/analytics/report?secret=SECRET&type=TYPE&period=7d
```

| Param | Required | Default | Values |
|-------|----------|---------|--------|
| `secret` | yes | -- | `ADMIN_SECRET` value |
| `type` | yes | -- | `traffic`, `article`, `funnel`, `flow`, `country` |
| `period` | no | `7d` | `Nd` format |

#### `type=traffic`

```json
{
  "period_days": 7,
  "total_page_views": 281,
  "unique_sessions": 142,
  "top_pages": [
    { "path": "/blog/compiler-ideas", "views": 95, "unique_sessions": 80 }
  ],
  "top_referrers": [
    { "referrer": "https://news.ycombinator.com/", "count": 34 }
  ],
  "daily_views": [
    { "date": "2026-03-23", "count": 41 }
  ]
}
```

#### `type=article`

```json
{
  "period_days": 7,
  "articles": [
    {
      "path": "/blog/compiler-ideas",
      "views": 95,
      "avg_scroll_depth": 72.5,
      "avg_visibility_seconds": 184.3,
      "bounce_rate": 45.2
    }
  ]
}
```

#### `type=funnel`

Referral conversion funnel: external referrer -> blog -> projects -> GitHub click.

```json
{
  "period_days": 7,
  "steps": [
    { "name": "External referral sessions", "count": 34, "pct_of_first": 100.0 },
    { "name": "Visited blog post", "count": 28, "pct_of_first": 82.4 },
    { "name": "Visited projects", "count": 5, "pct_of_first": 14.7 },
    { "name": "Clicked GitHub link", "count": 3, "pct_of_first": 8.8 }
  ]
}
```

#### `type=flow`

Page-to-page navigation transitions.

```json
{
  "period_days": 7,
  "transitions": [
    { "from_path": "/", "to_path": "/blog", "count": 12 },
    { "from_path": "/blog", "to_path": "/blog/compiler-ideas", "count": 8 }
  ]
}
```

#### `type=country`

```json
[
  { "country": "US", "views": 43 },
  { "country": "Unknown", "views": 12 }
]
```

### Client Events

```
POST /api/analytics/event
Content-Type: application/json
```

Receives client-side behavioral events. Session ID from `nb_sid` cookie.

#### Scroll depth

```json
{ "event_type": "scroll", "path": "/blog/article", "data": { "depth": 75 } }
```

Thresholds: 25, 50, 75, 100. Fires once per threshold per page load.

#### Outbound click

```json
{ "event_type": "outbound_click", "path": "/blog/article", "data": { "url": "https://github.com/...", "text": "View source" } }
```

#### Visibility time

```json
{ "event_type": "visibility", "path": "/blog/article", "data": { "seconds": 120 } }
```

Cumulative time with page in focus. Sent on `pagehide`.

Returns `204 No Content` on success, `400 Bad Request` if invalid.

## Server-Side Middleware

Tracks every non-static page request. Runs as Axum middleware before handlers.

**Skipped paths:** `/static/*`, `/health*`, `/api/analytics/*`, `/admin/*`, `/.well-known/*`, `/favicon.ico`, `/robots.txt`, `/sitemap.xml`, `/rss.xml`

**Bot filtering:** 30+ User-Agent patterns (googlebot, curl, python, etc.). Requests without User-Agent are treated as bots.

**Session cookie:** `nb_sid` -- HttpOnly, SameSite=Lax, 30min TTL, Secure in production.

**Country detection priority:**
1. Cloudflare `CF-IPCountry` header (excludes XX, T1, ZZ)
2. DB-IP Lite database lookup (nightly auto-update)
3. NULL (shown as "Unknown" in reports)

**IP hashing:** SHA-256 with daily rotating salt. Raw IPs never stored.

## Scheduled Reports

When `ANALYTICS_REPORT_SCHEDULE` is set, a background task runs on the configured interval:

1. Aggregates raw data to `analytics_monthly` tables (preserves long-term trends)
2. Deletes `page_events` and `client_events` older than `ANALYTICS_RETENTION_DAYS`
3. Removes orphaned sessions
4. Sends HTML email report via Resend API

## Database Tables

| Table | Purpose | Retention |
|-------|---------|-----------|
| `page_events` | Server-side navigation events | `ANALYTICS_RETENTION_DAYS` |
| `client_events` | Client-side behavioral events (scroll, clicks, visibility) | `ANALYTICS_RETENTION_DAYS` |
| `analytics_sessions` | Session metadata (entry page, referrer) | Cleaned with events |
| `geoip_ranges` | IP-to-country lookup (DB-IP Lite) | Refreshed nightly |
| `analytics_monthly` | Aggregated monthly page stats | Indefinite |
| `analytics_monthly_referrers` | Aggregated monthly referrer stats | Indefinite |

## Nginx Setup

Forward Cloudflare headers to the app:

```nginx
location / {
    proxy_pass http://127.0.0.1:3000;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_set_header CF-IPCountry $http_cf_ipcountry;
}
```

Cloudflare `set_real_ip_from` + `real_ip_header CF-Connecting-IP` restores the real client IP.
