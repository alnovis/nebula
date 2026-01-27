# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.21] - 2026-01-27

### Added

### Changed

### Fixed


## [0.2.21] - 2026-01-27

### Added
- Tags display on project cards (up to 3 tags next to status badge)
- Cover image support on project detail pages
- Share buttons on project detail pages
- Renumbered Fields Support section in proto-wrapper-plugin project page

### Changed
- Project detail pages now use same styling as blog posts
- Expanded "Redesigning My Blog" article with Critical CSS, CLS fixes, CDN fallback, Mermaid lazy loading sections
- Expanded Nebula project page with Performance Optimizations, SEO, Security, Development Workflow, Infrastructure sections
- Expanded Proto Wrapper Plugin page with Field Contracts and Renumbered Fields documentation

### Fixed
- Mermaid diagrams rendering (added `m.default.run()` call)
- Mermaid diagram centering and sizing


## [0.2.20] - 2026-01-27

### Added

### Changed

### Fixed


## [0.2.20] - 2026-01-27

### Added
- Release automation scripts (`scripts/release.sh`, `scripts/changelog-gen.sh`)
- CHANGELOG.md and RELEASE_NOTES.md
- Comprehensive README with full feature documentation

### Changed
- Expanded critical CSS to prevent Cumulative Layout Shift (CLS)
- Added `min-height` to `.section`, `.post-item.has-cover`, `.hero-subtitle`
- Added `list-style: none` to critical CSS for immediate marker removal
- Added mobile media query to critical CSS

### Fixed
- CLS issues on blog list page (`.post-list`, `.post-item`)
- CLS on hero section (`.hero-subtitle`)
- CLS on section elements (`.section`)
- List marker flash on page load


## [0.2.19] - 2025-01-27

### Added
- Favicon support with multiple sizes (16, 32, 48, 180, 192px)
- `/favicon.ico` route at root for better SEO
- Git pre-commit hook for `cargo fmt`
- CHANGELOG.md and RELEASE_NOTES.md
- Release automation script

### Changed
- Updated README.md with full feature documentation

## [0.2.18] - 2025-01-27

### Added
- Cloudinary CDN integration for images
- Critical CSS inlined in `<head>` for fast first paint
- CDN fallback mechanism for HTMX/Mermaid (jsdelivr → cdnjs → unpkg)
- Deferred CSS loading with `media="print" onload`
- Template partials system (`partials/critical-css.html`, `partials/scripts.html`)

### Fixed
- Site accessibility from Russia (DPI bypass with inline scripts)

## [0.2.17] - 2025-01-26

### Added
- Cover images for blog posts and projects
- Share buttons (Twitter, LinkedIn, copy link)
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

### Fixed
- Various formatting issues

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

[Unreleased]: https://github.com/alnovis/nebula/compare/v0.2.21...HEAD
[0.2.21]: https://github.com/alnovis/nebula/compare/v0.2.21...v0.2.21
[0.2.21]: https://github.com/alnovis/nebula/compare/v0.2.20...v0.2.21
[0.2.20]: https://github.com/alnovis/nebula/compare/v0.2.20...v0.2.20
[0.2.20]: https://github.com/alnovis/nebula/compare/v0.2.19...v0.2.20
[0.2.19]: https://github.com/alnovis/nebula/compare/v0.2.18...v0.2.19
[0.2.18]: https://github.com/alnovis/nebula/compare/v0.2.17...v0.2.18
[0.2.17]: https://github.com/alnovis/nebula/compare/v0.2.10...v0.2.17
[0.2.10]: https://github.com/alnovis/nebula/compare/v0.2.9...v0.2.10
[0.2.9]: https://github.com/alnovis/nebula/compare/v0.2.7...v0.2.9
[0.2.7]: https://github.com/alnovis/nebula/compare/v0.2.0...v0.2.7
[0.2.0]: https://github.com/alnovis/nebula/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/alnovis/nebula/releases/tag/v0.1.0
