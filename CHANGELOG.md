# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/alnovis/nebula/compare/v0.2.30...HEAD
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
