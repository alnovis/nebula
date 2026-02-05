# Release Notes

## v0.2.30

**Release Date:** 2026-02-05

### Social Media & RSS

- **Open Graph image dimensions** — Added `og:image:width` (1200), `og:image:height` (630), and `og:image:alt` for correct preview rendering on Facebook, LinkedIn, Telegram
- **Twitter Card upgrade** — Automatically uses `summary_large_image` when cover image is present for larger previews
- **RSS feed improvements** — Added GUID (permalink) and author fields for proper feed reader deduplication and attribution

### Migration

No breaking changes.

---

## v0.2.29

**Release Date:** 2026-02-05

### SEO Improvements

- **JSON-LD Structured Data** — Article schema for blog posts, Article + SoftwareSourceCode for projects. Enables rich snippets in Google search results.

- **Tag Pages** — New `/blog/tag/{tag}` routes allow users to browse posts by topic. Tags are now clickable throughout the site. All tag pages are included in sitemap.xml.

### New Features

- `all_tags()` method returns all unique tags across published posts
- `posts_by_tag(tag)` method filters posts by tag (case-insensitive)

### Migration

No breaking changes. Tag pages are automatically generated from existing post tags.

---

## v0.2.27

**Release Date:** 2026-02-05

### Content

- New blog post: "The Architecture of Modern Compilers: MLIR, Nanopass, and Green-Red Trees"
- Local image support for development environment

---

## v0.2.26

**Release Date:** 2026-01-29

### Bug Fixes

- Project cards layout — footer now pinned to bottom using flexbox

---

## Previous Releases

See [CHANGELOG.md](CHANGELOG.md) for full version history.
