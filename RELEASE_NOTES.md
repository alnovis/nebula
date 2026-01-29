# Release Notes

## v0.2.25

**Release Date:** 2026-01-29

### Highlights

- Restructured CI/CD pipeline with proper job dependencies
- Improved project cards layout with views counter on separate line

### New Features

- **GitHub Releases** — automated release creation with changelog extraction
- **Health check** — deployment now verifies site is up after deploy
- **Redis for development** — docker-compose.yml now includes Redis service

### Improvements

- **CI/CD Pipeline** — split into `build.yml` (for PRs) and `release.yml` (for tags)
- **Release workflow** — proper job dependencies: validate → build/upload → release → deploy
- **Project cards** — views counter moved to separate line for cleaner layout

### Bug Fixes

- Fixed views counter alignment with status badge in project cards

### Migration

Add `REDIS_URL=redis://localhost:6379` to your `.env` file for local development with Redis.
Docker users: run `docker compose up -d` to start Redis alongside PostgreSQL.

---

## Previous Releases

See [CHANGELOG.md](CHANGELOG.md) for full version history.
