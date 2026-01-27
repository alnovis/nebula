# Release Notes

## v0.2.21

**Release Date:** 2026-01-27

### Highlights

This release adds CDN diagnostics for monitoring accessibility from Russia, improves project pages with tags and consistent styling, and significantly expands documentation content.

### New Features

- **CDN Diagnostics** — New `/health/russia` endpoint serves a diagnostic page that tests CDN availability from the client's browser. Tests jsdelivr, cdnjs, unpkg, and Google Fonts. Results can be sent to server for logging via `/health/russia/report`.

- **Tags on project cards** — Up to 3 technology tags now appear next to the status badge on both homepage and projects list, helping visitors quickly identify relevant projects.

- **Project detail page improvements** — Cover images and share buttons now appear on project pages, matching blog post styling.

### Improvements

- **Projects list page** — Tags moved from bottom to next to status badge for consistency with homepage.

- **Expanded documentation content:**
  - "Redesigning My Blog" article expanded with sections on Critical CSS, CLS optimization, CDN fallback for Russia, Mermaid lazy loading, and favicon creation journey
  - Nebula project page expanded with Performance Optimizations, SEO & Social, Security, Development Workflow, and Infrastructure sections
  - Proto Wrapper Plugin page enhanced with Field Contracts documentation and Renumbered Fields Support section

### Bug Fixes

- Fixed Mermaid diagrams not rendering (missing `run()` call after initialization)
- Fixed Mermaid diagram centering and sizing in dark theme

### Migration

No breaking changes.

---

## Previous Releases

See [CHANGELOG.md](CHANGELOG.md) for full version history.
