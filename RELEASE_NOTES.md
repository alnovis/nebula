# Release Notes

## v0.2.20

**Release Date:** 2026-01-27

### Highlights

This release focuses on fixing Cumulative Layout Shift (CLS) issues and adding release automation tooling.

### New Features

- **Release automation** — `scripts/release.sh` for version bumping, changelog updates, and tagging
- **Changelog generator** — `scripts/changelog-gen.sh` to generate changelog entries from git commits
- **Documentation** — Comprehensive README with full feature documentation, project structure, and deployment guide

### Improvements

- **Critical CSS expanded** — More styles inlined to prevent layout shifts during page load
- **CLS optimizations**:
  - `min-height` on sections and post items reserves space before content loads
  - `list-style: none` applied immediately to prevent marker flash
  - Grid layout for post covers defined in critical CSS
  - Mobile breakpoint added to critical CSS

### Bug Fixes

- Fixed CLS on blog list page (`.post-list`, `.post-item`) — score improved from 0.47 to <0.1
- Fixed CLS on hero section (`.hero-subtitle`) — score improved from 0.53 to <0.1
- Fixed CLS on main page sections (`.section`) — score improved from 0.74 to <0.1
- Fixed list marker appearing briefly before being hidden

### Migration

No breaking changes.

---

## Previous Releases

See [CHANGELOG.md](CHANGELOG.md) for full version history.
