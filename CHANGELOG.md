# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- CI runs on `ubuntu-24.04` instead of `ubuntu-latest`, which moves to Ubuntu 26.04 by 2026-11-19; the figures job relies on the preinstalled Chrome and on apt font packages
- Docker and release actions move to their Node 24 majors: setup-buildx v4, login v4, metadata v6, build-push v7, action-gh-release v3

### Fixed
- `og:image` and `twitter:image` of posts and projects point to a PNG rendered by Cloudinary (`f_png`) instead of the SVG cover. LinkedIn, Facebook and X do not render SVG, so shared links had no image

## [0.2.49] - 2026-11-11

### Added
- Analytics dashboard: "Unconfirmed" column in Article Performance. It counts sessions on a post that sent no client-side events but look like a person: a current browser user agent and a referrer from outside the site. EasyPrivacy, on by default in uBlock Origin and Brave, blocks `/api/analytics/event`, so these readers never show up as readers. The count is an upper bound, since some bots pass the same check

- Analytics dashboard: one selector next to the period narrows the page to a post or another page (`/`, `/blog`, `/projects`, `/about`, `/resume`, `/resume/print`, `/contact`); "All" is the default and shows the whole site
- Summary tiles for the selected scope: page views, readers, unconfirmed, share of visits gone in under 30 s, reads of 5 minutes or more, median visible time
- Reading Depth for a selected page: readers who scrolled to 25, 50, 75 and 100 percent, and how many reached the end in under 5 minutes
- Post Funnel for a selected post: readers, then those who visited projects or clicked a GitHub link after their first load of the post. The "-> Proj" and "-> GH" columns of Article Performance count any visit in the period, before or after

### Changed
- Top Referrers becomes Sources, full width: referrer hosts with readers, unconfirmed sessions, share gone in under 30 s, reads of 5 minutes or more and median time. Under "All" a source is the referrer of a session's first load, for a selected page the referrer of its first load of that page; direct visits are listed too
- Article Trend becomes Trend and follows the selector; under "All" it sums the readers of all posts
- Countries follow the selector; Top Pages moves below them and is shown under "All" only; Article Performance is shown under "All" only
- Daily cleanup keeps sessions without client-side events that pass the same check instead of deleting them as bots; the rest are still deleted
- Funnel steps "blog" and "projects" and the monthly aggregation count only sessions with client-side events. They relied on the cleanup to remove the others, which now stay

### Removed
- Analytics dashboard: Daily Views chart; Trend under "All" covers the posts, and Top Pages the rest

## [0.2.48] - 2026-11-11

### Changed
- Resume: "Selected Work" becomes "Open Source & Projects". Proto Wrapper Plugin and Nebula get one-line descriptions, and the TrustTunnel entry is replaced by one item listing contributions to TrustTunnel (Rust), TrustTunnelClient (C++) and Eclipse JKube (Java). Web and print versions match

## [0.2.47] - 2026-11-10

### Fixed
- Article trend: the current hour or day was drawn only as a projection, so the chart looked empty until a dashed line shot up; the lines now end at the count so far
- Article trend projection scaled the count so far by the share of the bucket that had passed, so a spike early in the day projected the whole day at that rate; it now adds the last hour's rate for the time left
- Version is 0.2.47 in `Cargo.toml`; v0.2.46 was tagged without the bump and reports itself as 0.2.45

## [0.2.46] - 2026-11-10

### Added
- Analytics dashboard: article trend chart for one post, the newest by default, with a selector for any published post. It draws readers and readers who scrolled halfway as smoothed lines, hourly for the new `2d` period and daily otherwise, and shows the current hour or day as a dashed projection of the rate so far

### Changed
- Posts "An Aggregate Is a Function, Not a Sum" and "Phoenix vs ClickHouse: Why the Argument Keeps Coming Back" are published again with the edits from 0.2.45; proofreading changes will follow separately

### Removed
- Watchtower from `docker-compose.prod.yml`: it crash-looped on a Docker API it no longer supports, and the release workflow already deploys every tag

## [0.2.45] - 2026-11-10

### Added
- Analytics leaves out the owner's visits: opening the dashboard sets an owner cookie, and `ANALYTICS_EXCLUDE_IPS` (server `.env`) lists addresses to skip, matched against every hop of the forwarding chain
- `scripts/analytics-exclude-ips.sql` deletes sessions recorded from given addresses; dry run unless `-v apply=1`
- Article performance shows readers (sessions), median time and reads of 5 minutes or more

### Changed
- Article metrics other than views are counted per session, so reloads no longer weigh a session more; the JSON `article` report fields follow (`readers`, `median_time`, `long_reads` instead of `avg_time`)
- Visible time is also sent when the page is hidden, not only on `pagehide`, which mobile browsers often skip
- Posts "An Aggregate Is a Function, Not a Sum" and "Phoenix vs ClickHouse" are back in drafts for proofreading; both are edited for language and checked against the stand results, and the Phoenix post is retitled "Phoenix vs ClickHouse: Why the Argument Keeps Coming Back" with a new cover (`phoenix-vs-clickhouse-cover-v3.svg`)

## [0.2.44] - 2026-11-09

### Added
- Figures are rendered in two sets: dark as on the site, and light (`-light.png`) for white pages; the light palette keeps small text at 5.5:1 or more on white

### Changed
- RSS feed links the light figures, which suit the dev.to RSS import and most feed readers
- `scripts/export-devto.mjs` takes `--theme light|dark`, light by default

## [0.2.43] - 2026-11-09

### Added
- `scripts/publish-figures.mjs`: renders inline SVG figures of all posts to PNG in headless Chrome and uploads the missing ones to Cloudinary; runs in the content and release workflows

### Changed
- RSS feed replaces inline SVG figures with those PNGs and drops the figure stylesheet, so feed readers and the dev.to RSS import show the figures
- `scripts/export-devto.mjs` links figures to the published PNGs; rendering code shared in `scripts/figures.mjs`

## [0.2.42] - 2026-11-09

### Added
- Blog post "Phoenix vs ClickHouse: Why the Fight Never Ends" with its SVG cover
- `scripts/export-devto.mjs`: exports a post for dev.to, rendering inline SVG figures to PNG and converting `<details>` and site-relative links

## [0.2.41] - 2026-11-04

### Added
- `docs/VISUAL_STYLE.md`: colours with measured contrast, typography, blocks, arrows, templates and a checklist for covers and in-article figures

### Changed
- Post and project covers redrawn to the visual style: readable text contrast, one font for all text, consistent line weights, arrows and rounded blocks; published under `-v2` names so cached copies are not reused
- About page: experience updated to over 20 years

## [0.2.40] - 2026-11-04

### Added
- Blog post "An Aggregate Is a Function, Not a Sum" with its SVG cover

### Changed
- Content guide describes hand-crafted SVG covers instead of WebP
- Content sorting uses `sort_by_key` (clippy `unnecessary_sort_by`)

### Fixed
- Reading time counts only rendered text; inline HTML such as SVG figures no longer inflates it

## [0.2.39] - 2026-03-30

### Changed
- Article Performance: split conversion into `→ Proj` and `→ GH` columns
- Avg Time formatted as `Xm Ys` instead of raw seconds

## [0.2.38] - 2026-03-30

### Changed
- Article Performance: bounce rate replaced with conversion % (sessions → projects/GitHub)
- Article Performance: added avg scroll, max scroll, avg time columns
- Reports: all queries use CTE JOIN for human session filtering (replaces EXISTS subqueries)
- Article report: N+1 queries (151) → single query with CTEs
- Monthly aggregation: pre-aggregated scroll/visibility CTEs (fixes NULL averaging)
- New index: `client_events(session_id, event_type)`

### Fixed
- Bounce rate formula corrected (% sessions with single page view)
- IPv6 referrer detection in `is_ip_referrer`
- `KNOWN_ROUTES_FILTER` aligned with middleware whitelist

## [0.2.37] - 2026-03-30

### Changed
- Daily Views chart rewritten as inline SVG (pixel-perfect axes, no CSS layout hacks)
- Analytics middleware: whitelist-based tracking (only known routes, scanners ignored)
- Reports filter by verified human sessions (JS executed = real browser, CTE JOIN)
- Article report: N+1 queries (151) → single query with CTEs
- Auto-cleanup bot sessions (no client JS events after 5 min = bot)
- Daily cleanup runs independently of email report schedule
- Referral Funnel: increased bar contrast (`opacity` 0.15 → 0.35)

### Fixed
- GeoIP: filter ZZ (reserved/private) entries from DB-IP import, forward Cloudflare `CF-IPCountry` header through Nginx
- Top Referrers: filter out self-referrers and IP-address referrers (IPv4 + IPv6)
- Bounce rate: corrected formula (% sessions with single page view)
- Monthly aggregation: fix scroll/visibility averaging with pre-aggregated CTEs

## [0.2.36] - 2026-03-29

### Fixed
- GeoIP country detection: use Cloudflare `CF-IPCountry` header as primary source, DB-IP as fallback (fixes 75%+ ZZ entries)
- Referral funnel now filters internal referrers (own domain excluded from external session count)
- Navigation Flow table displayed at full width for readability
- Daily Views chart: all days in period shown (including zero-traffic days), Y-axis grid lines with labels
- Referrer URLs expand on hover instead of truncating
- Analytics middleware skips `/.well-known/` and `/admin` paths

### Changed
- Referral Funnel and Countries sections moved to two-column grid layout
- load-env.fish and load-env.sh rewritten for correct variable parsing

## [0.2.35] - 2026-03-29

### Added
- Behavioral analytics with server-side middleware (session tracking, page navigation events)
- Client-side tracking: scroll depth, outbound link clicks, visibility time
- GeoIP country detection via DB-IP Lite database (auto-updated nightly)
- Analytics admin dashboard (`/admin/analytics`) with daily views chart, top pages, referrers, article performance, referral funnel, country breakdown, navigation flow
- JSON analytics API (`/admin/analytics/report`) for programmatic access
- Scheduled analytics email reports (weekly/daily via Resend API)
- Monthly data aggregation before cleanup (preserves historical trends)
- Automatic cleanup of raw analytics data older than retention period (default 90 days)
- `send_html_email()` method on EmailService
- Standalone `is_bot()` and `hash_ip_daily()` utility functions

### Changed
- Bot detection extracted from ViewsService to standalone public function
- docker-compose.prod.yml now passes analytics environment variables
- load-env.fish and load-env.sh rewritten for correct variable parsing

## [0.2.34] - 2026-03-28

### Added
- About page (`/about`) with bio, projects, and external links
- CV print version (`/resume/print`) — white theme, URLs as text, optimized for PDF export
- "Print version" button on CV page
- CSS cache busting via `?v={{ version }}` query param on stylesheet

### Changed
- CV fully rewritten: achievement-based format with metrics (1.5M devices, p99 latency 100ms→5ms, petabyte-scale HyperLogLog)
- CV skills split into Core and Also worked with
- CV education and certifications compacted to single lines
- CV removed location references from all experience entries
- Reading time delimiter: now framed with dots on both sides (`· 7 min read ·`)
- Removed views count leading dot separator
- Resume hidden from top navigation (accessible via About page)

## [0.2.33] - 2026-03-28

### Changed
- SVG cover viewBox tightened for better content-to-container alignment
- Project cards: status badge moved to title row, footer simplified to GitHub + tags + views
- Post metadata: grid layout with views counter pinned to the right, tags wrap independently
- Project cover SVGs: increased icon opacity, refined typography and layout
- Removed unused webp cover images

### Fixed
- Cloudinary URL resolution strips file extension (Cloudinary serves by public_id)
- Project card footer no longer breaks across lines on narrow screens

## [0.2.32] - 2026-03-28

### Added
- Hand-crafted SVG cover images for all blog posts and projects
- Inline tags in post metadata on index page
- View counts on index page posts via Redis
- Hover border effect on cover images and tags

### Changed
- Cover images switched from webp to SVG across all posts and projects
- Post list layout: full-width covers instead of side-by-side grid
- Reduced section and hero spacing for tighter layout
- Doubled article splitter margins for better visual separation
- Post title hidden when cover image is present (title baked into SVG)

### Fixed
- CI/CD upload-media job now uploads SVG files alongside webp to Cloudinary

## [0.2.30] - 2026-02-05

### Added
- Open Graph image dimensions (width, height, alt) for better social media previews
- GUID and author fields in RSS feed for proper feed reader support
- Twitter Card switches to `summary_large_image` when cover image is present

## [0.2.29] - 2026-02-05

### Added
- Tag pages `/blog/tag/{tag}` with filtered post list
- Clickable tags in blog list, post page, and tag pages
- Tags included in sitemap.xml for indexing
- JSON-LD structured data for blog posts (Article schema)
- JSON-LD structured data for projects (Article + SoftwareSourceCode schemas)
- `all_tags()` and `posts_by_tag()` methods in ContentStore

### Changed
- Tags are now links to tag pages throughout the site

## [0.2.27] - 2026-02-05

### Added
- "The Architecture of Modern Compilers" blog post
- Local image support in development environment
- Cloudinary image resolution based on environment

## [0.2.26] - 2026-01-29

### Fixed
- Project card footer now pinned to bottom using flexbox, ensuring consistent alignment across all cards

## [0.2.25] - 2026-01-29

### Added
- Redis service to development docker-compose.yml
- REDIS_URL configuration to .env and .env.example
- Health check step in deployment workflow
- GitHub Release creation in CI/CD pipeline

### Changed
- Restructured CI/CD: split into build.yml (CI) and release.yml (deployment)
- Release pipeline now has proper job dependencies: validate → build-docker/upload-media → create-release → deploy
- Views counter moved to separate line in project cards for better layout
- Project cards now use project-footer wrapper for consistent alignment

### Fixed
- Views counter alignment with status badge in project cards

## [0.2.24] - 2026-01-29

### Added
- Views counter for blog posts and projects with unique visitor tracking
- Redis integration for views storage (optional, graceful degradation)
- Bot detection via User-Agent filtering
- Eye icon with view count display on single pages and list pages
- Batch view count fetching for list pages (MGET)
- Privacy-preserving IP hashing (SHA256, no raw IPs stored)

### Changed
- docker-compose.prod.yml now includes Redis service

## [0.2.23] - 2026-01-27

### Fixed
- CDN diagnostics - test CSS resources via link element

## [0.2.22] - 2026-01-27

### Added
- CDN diagnostics endpoint `/health/cdn` for testing CDN availability
- CDN report endpoint `/health/cdn/report` with logging for blocked resources
- Tags display on project cards (up to 3 tags next to status badge)

### Fixed
- Mermaid diagrams rendering

## [0.2.21] - 2026-01-27

### Added
- Cover image support on project detail pages
- Share buttons on project detail pages
- Expanded documentation content

### Changed
- Project detail pages now use same styling as blog posts
- Projects list page: tags moved next to status badge

### Fixed
- Mermaid diagram centering and sizing

## [0.2.20] - 2026-01-27

### Added
- Release automation scripts
- CHANGELOG.md and RELEASE_NOTES.md
- Comprehensive README with full feature documentation

### Changed
- Expanded critical CSS to prevent Cumulative Layout Shift (CLS)

### Fixed
- CLS issues on blog list page, hero section, section elements
- List marker flash on page load

## [0.2.19] - 2025-01-27

### Added
- Favicon support with multiple sizes (16, 32, 48, 180, 192px)
- `/favicon.ico` route at root for better SEO
- Git pre-commit hook for `cargo fmt`

## [0.2.18] - 2025-01-27

### Added
- Cloudinary CDN integration for images
- Critical CSS inlined in `<head>` for fast first paint
- CDN fallback mechanism for HTMX/Mermaid (jsdelivr → cdnjs → unpkg)
- Deferred CSS loading with `media="print" onload`

### Fixed
- Site accessibility from Russia (DPI bypass with inline scripts)

## [0.2.17] - 2025-01-26

### Added
- Cover images for blog posts and projects
- Share buttons (Twitter, LinkedIn, Telegram)
- Reading time estimation
- Back-to-top button
- WebP image optimization

### Changed
- Blog visual redesign
- Hero section gradient improvements

## [0.2.10] - 2025-01-25

### Added
- Responsive header logo (full name on desktop, "AN" on mobile)

## [0.2.9] - 2025-01-24

### Added
- Content sync in GitHub Actions workflow
- Updated blog articles

## [0.2.7] - 2025-01-23

### Added
- Admin endpoint for hot content reload (`/admin/reload`)
- Open Graph meta tags
- Twitter Card meta tags
- Cloudflare Web Analytics
- robots.txt

## [0.2.0] - 2025-01-20

### Added
- Blog with Markdown support
- Projects showcase
- Resume/CV page
- Contact form with Turnstile captcha
- Email integration via Resend
- RSS feed generation
- Sitemap generation
- Syntax highlighting with Syntect
- Mermaid diagrams support
- Docker deployment
- GitHub Actions CI/CD
- Traefik integration

## [0.1.0] - 2025-01-15

### Added
- Initial release
- Basic Axum setup
- Askama templates
- PostgreSQL integration

[Unreleased]: https://github.com/alnovis/nebula/compare/v0.2.49...HEAD
[0.2.49]: https://github.com/alnovis/nebula/compare/v0.2.48...v0.2.49
[0.2.48]: https://github.com/alnovis/nebula/compare/v0.2.47...v0.2.48
[0.2.47]: https://github.com/alnovis/nebula/compare/v0.2.46...v0.2.47
[0.2.46]: https://github.com/alnovis/nebula/compare/v0.2.45...v0.2.46
[0.2.45]: https://github.com/alnovis/nebula/compare/v0.2.44...v0.2.45
[0.2.44]: https://github.com/alnovis/nebula/compare/v0.2.43...v0.2.44
[0.2.43]: https://github.com/alnovis/nebula/compare/v0.2.42...v0.2.43
[0.2.42]: https://github.com/alnovis/nebula/compare/v0.2.41...v0.2.42
[0.2.41]: https://github.com/alnovis/nebula/compare/v0.2.40...v0.2.41
[0.2.40]: https://github.com/alnovis/nebula/compare/v0.2.39...v0.2.40
[0.2.39]: https://github.com/alnovis/nebula/compare/v0.2.38...v0.2.39
[0.2.38]: https://github.com/alnovis/nebula/compare/v0.2.37...v0.2.38
[0.2.37]: https://github.com/alnovis/nebula/compare/v0.2.36...v0.2.37
[0.2.36]: https://github.com/alnovis/nebula/compare/v0.2.35...v0.2.36
[0.2.35]: https://github.com/alnovis/nebula/compare/v0.2.34...v0.2.35
[0.2.34]: https://github.com/alnovis/nebula/compare/v0.2.33...v0.2.34
[0.2.33]: https://github.com/alnovis/nebula/compare/v0.2.32...v0.2.33
[0.2.32]: https://github.com/alnovis/nebula/compare/v0.2.30...v0.2.32
[0.2.30]: https://github.com/alnovis/nebula/compare/v0.2.29...v0.2.30
[0.2.29]: https://github.com/alnovis/nebula/compare/v0.2.27...v0.2.29
[0.2.27]: https://github.com/alnovis/nebula/compare/v0.2.26...v0.2.27
[0.2.26]: https://github.com/alnovis/nebula/compare/v0.2.25...v0.2.26
[0.2.25]: https://github.com/alnovis/nebula/compare/v0.2.24...v0.2.25
[0.2.24]: https://github.com/alnovis/nebula/compare/v0.2.23...v0.2.24
[0.2.23]: https://github.com/alnovis/nebula/compare/v0.2.22...v0.2.23
[0.2.22]: https://github.com/alnovis/nebula/compare/v0.2.21...v0.2.22
[0.2.21]: https://github.com/alnovis/nebula/compare/v0.2.20...v0.2.21
[0.2.20]: https://github.com/alnovis/nebula/compare/v0.2.19...v0.2.20
[0.2.19]: https://github.com/alnovis/nebula/compare/v0.2.18...v0.2.19
[0.2.18]: https://github.com/alnovis/nebula/compare/v0.2.17...v0.2.18
[0.2.17]: https://github.com/alnovis/nebula/compare/v0.2.10...v0.2.17
[0.2.10]: https://github.com/alnovis/nebula/compare/v0.2.9...v0.2.10
[0.2.9]: https://github.com/alnovis/nebula/compare/v0.2.7...v0.2.9
[0.2.7]: https://github.com/alnovis/nebula/compare/v0.2.0...v0.2.7
[0.2.0]: https://github.com/alnovis/nebula/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/alnovis/nebula/releases/tag/v0.1.0
