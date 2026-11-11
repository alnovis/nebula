# Architecture

Nebula is one Axum binary with three stores beside it: Markdown files on disk,
PostgreSQL for analytics, and Redis for view counters and session state. Pages
are rendered on the server from compiled Askama templates.

## Table of Contents

- [Stack](#stack)
- [Startup](#startup)
- [Modules](#modules)
- [Request Path](#request-path)
- [Content](#content)
- [Images](#images)
- [Feeds](#feeds)
- [Frontend](#frontend)
- [Database](#database)
- [Production Layout](#production-layout)

---

## Stack

| Component | Technology |
|-----------|------------|
| Language | Rust, edition 2021 (the Docker image builds with Rust 1.93) |
| Web framework | Axum 0.7, tower-http 0.5 (static files, gzip, tracing) |
| Templates | Askama 0.12, checked at compile time |
| Markdown | pulldown-cmark 0.10, Syntect 5 for code highlighting |
| Database | PostgreSQL 16 through SQLx 0.7, migrations run at startup |
| Cache | Redis 7 (optional) |
| HTTP client | reqwest 0.12 with rustls: Resend, Turnstile, DB-IP downloads |
| Client side | HTMX 1.9, Mermaid 10, a small analytics script |
| Images | Cloudinary (covers, figure PNGs) |
| Build and deploy | Docker multi-stage Alpine build, GHCR, GitHub Actions |
| Edge | Cloudflare in front of nginx (`deploy/nginx-alnovis.io.conf`) |

## Startup

`create_app` in `src/lib.rs` runs in this order:

1. Connects to PostgreSQL and applies `migrations/`.
2. Loads every post and project from `CONTENT_DIR` into a `ContentStore`.
3. Connects to Redis if `REDIS_URL` is set. A failed connection only disables
   view counters and session state; the site still starts.
4. Starts two background tasks: the DB-IP country database updater and the
   analytics scheduler (aggregation, cleanup, email reports).
5. Builds the router and wraps it in middleware.

Without `DATABASE_URL` the process does not start.

## Modules

```
src/
  main.rs            entry point: .env, tracing, bind 0.0.0.0:PORT
  lib.rs             create_app: startup and the router
  config.rs          Config from environment variables, cover URL resolution
  state.rs           AppState: pool, ContentStore behind RwLock, config, email, Redis
  email.rs           contact mail through the Resend API
  turnstile.rs       Cloudflare Turnstile verification
  views.rs           view counters in Redis, bot detection, IP hashing
  content/
    mod.rs           ContentStore: posts, projects, tags, drafts filtered out
    markdown.rs      frontmatter, Markdown to HTML, Syntect, Mermaid blocks
    feed.rs          post HTML for RSS: inline SVG figures become PNG links
  models/            Post and Project with their frontmatter
  analytics/
    middleware.rs    server-side page events, exclusions
    events.rs        page events and client events (scroll, visibility, outbound click)
    session.rs       session cookie, session rows, session state in Redis
    exclude.rs       ANALYTICS_EXCLUDE_IPS and the owner cookie
    geoip.rs         country lookup and the DB-IP updater
    reports.rs       dashboard and report queries
    trend.rs         trend series for the dashboard
    scheduler.rs     aggregation, retention cleanup, email reports
  routes/            one file per section: pages, blog, projects, about, resume,
                     contact, feeds, analytics, admin, health
templates/           Askama templates; partials/ holds critical CSS and scripts
static/              CSS, JS, favicons, cover SVGs for local development
migrations/          SQLx migrations
scripts/             figures, dev.to export, Cloudinary upload, analytics SQL
```

## Request Path

Middleware runs outside in: `TraceLayer`, then gzip `CompressionLayer`, then the
analytics middleware, then the route handler.

The analytics middleware skips static files, health checks, feeds, admin and
API paths. For a page it reads or sets the `nb_sid` session cookie, takes the
previous path from Redis, runs the handler, and then records the page event in a
spawned task, so the response does not wait for PostgreSQL. Details are in
[ANALYTICS.md](ANALYTICS.md).

Handlers take a read lock on the `ContentStore`, render an Askama template and
return HTML. The blog post handler also bumps and reads the view counter when
Redis is available.

## Content

Content lives in `content/blog/*.md` and `content/projects/*.md`, each file with
YAML-like frontmatter. Fields are described in [CONTENT_GUIDE.md](CONTENT_GUIDE.md).

- Markdown is rendered once, at load time, with tables, footnotes,
  strikethrough and task lists. Fenced code is highlighted by Syntect;
  a `mermaid` block becomes `<pre class="mermaid">` for the client to draw.
  Raw HTML, such as inline SVG figures, passes through.
- Reading time counts text and code words only, at 200 words per minute.
- A post with `draft: true` is left out of lists and tags and returns 404.
- `POST /admin/reload?secret=...` loads the directory again and swaps the whole
  store under the write lock. CI calls it after copying `content/` to the server.

## Images

A cover is named in frontmatter by file name. `Config::resolve_cover_url` maps
it to `/static/images/<name>` in development and to
`<CLOUDINARY_BASE_URL>/<name without extension>` in production; the page shows
the SVG. `Config::resolve_og_image_url` asks Cloudinary for a PNG (`f_png`) for
Open Graph and Twitter Card tags, because link previews do not render SVG.

Covers reach Cloudinary through `release.yml`. Figure PNGs are rendered by
`scripts/publish-figures.mjs` with headless Chrome and uploaded under
`nebula/figures/<slug>-<hash>[-light].png`.

## Feeds

- `/rss.xml` carries full post HTML. `content/feed.rs` removes the figure
  stylesheet and shared `<defs>`, and replaces each `<figure><svg>` with the light
  PNG of that figure. The name is derived from a hash of the SVG markup, the
  same way `scripts/figures.mjs` names the uploads.
- `/sitemap.xml` lists pages, posts, projects and tag pages.
- `/robots.txt` points to the sitemap.

## Frontend

- `templates/partials/critical-css.html` is inlined in `<head>`. The full
  `static/css/style.css` loads deferred and carries `?v=<version>` so a release
  invalidates caches.
- `templates/partials/scripts.html` loads HTMX from jsDelivr, then cdnjs, then
  unpkg, whichever answers first. Mermaid is imported only when the page has a
  `.mermaid` block.
- `templates/partials/analytics.html` holds the client analytics script.

## Database

PostgreSQL holds analytics only: sessions, page events, client events, monthly
aggregates and the GeoIP table. Content never touches it. Migrations in
`migrations/` are applied by `sqlx::migrate!` at startup. An applied migration
is never edited; a change is a new migration.

Redis holds view counters (unique per hashed IP) and the short session state
used for the previous-path field of page events. Both are optional.

## Production Layout

```
Cloudflare -> nginx (TLS, CF-IPCountry) -> nebula-app :3000
                                            |-- nebula-db    (postgres:16-alpine)
                                            `-- nebula-redis (redis:7-alpine)
```

The stack is `docker-compose.prod.yml` in `/opt/nebula` on the server, with
`content/` mounted read-only into the app container. Deployment steps are in
[DEVELOPMENT.md](DEVELOPMENT.md#deployment).
