# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.37] - 2026-03-29

### Fixed
- GeoIP: filter ZZ (reserved/private) entries from DB-IP import, forward Cloudflare `CF-IPCountry` header through Nginx
- Top Referrers: filter out self-referrers (own domain)
- Daily Views chart: X-axis labels no longer overlap at period boundaries or overflow the chart
- Referral Funnel: increased bar contrast (`opacity` 0.15 → 0.35)

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

[Unreleased]: https://github.com/alnovis/nebula/compare/v0.2.37...HEAD
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
