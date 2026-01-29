# Release Notes

## v0.2.24

**Release Date:** 2026-01-29

### Highlights

Views counter feature - track unique page views on blog posts and projects with Redis backend.

### New Features

- **Views Counter** - Display view counts on blog posts and projects
  - Unique visitor tracking by IP hash (privacy-preserving)
  - Bot detection via User-Agent filtering
  - Eye icon with formatted count ("1.2k views")
  - Works on single pages and list pages
  - Batch fetching with Redis MGET for list pages

- **Redis Integration** - Optional Redis backend for views storage
  - Graceful degradation when Redis unavailable
  - Persistent storage with AOF enabled
  - Configurable via `REDIS_URL` environment variable

### Improvements

- docker-compose.prod.yml now includes Redis service with persistent volume

### Bug Fixes

None

### Migration

**Optional:** To enable views counter, add Redis:

```yaml
# docker-compose.prod.yml already updated
# Just redeploy and views will start counting
```

If Redis is not available, the site works normally without view counts.

---

## v0.2.23

**Release Date:** 2026-01-27

### Previous Release

See [CHANGELOG.md](CHANGELOG.md) for details.

---

## Previous Releases

See [CHANGELOG.md](CHANGELOG.md) for full version history.
