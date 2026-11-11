# Development

How to run Nebula locally, configure it, check a change and ship it.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Running Locally](#running-locally)
- [Configuration](#configuration)
- [Checks](#checks)
- [Figures and dev.to Export](#figures-and-devto-export)
- [CI Workflows](#ci-workflows)
- [Releasing](#releasing)
- [Deployment](#deployment)

---

## Prerequisites

- Rust, stable toolchain. The Docker image builds with Rust 1.93.
- Docker, for PostgreSQL and Redis.
- Node.js and Google Chrome, only for the figure and dev.to scripts.

## Running Locally

```bash
docker compose up -d    # PostgreSQL 16 on :5432, Redis 7 on :6379
cp .env.example .env
cargo run               # http://localhost:3000
```

`.env` is read at startup. Migrations run on every start. Covers are served from
`static/images/` unless `ENVIRONMENT=production`.

Content is read once at startup. To pick up an edit without a restart, set
`ADMIN_SECRET` and call:

```bash
curl -X POST "http://localhost:3000/admin/reload?secret=$ADMIN_SECRET"
```

## Configuration

All settings are environment variables. Only `DATABASE_URL` is required.

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | -- | PostgreSQL connection string |
| `PORT` | `3000` | Listening port; the server binds `0.0.0.0` |
| `CONTENT_DIR` | `./content` | Directory with `blog/` and `projects/` |
| `ENVIRONMENT` | `development` | `production` serves covers from Cloudinary and marks cookies `Secure` |
| `SITE_URL` | `http://localhost:3000` | Base for canonical URLs, feeds and the sitemap |
| `SITE_TITLE` | `Nebula` | Home page title, RSS channel title, subject prefix of contact mail |
| `SITE_DESCRIPTION` | `Personal blog and project showcase` | RSS channel description |
| `AUTHOR_NAME` | `Author` | Name on About, resume, post and project pages |
| `AUTHOR_EMAIL` | `author@example.com` | RSS item author, address on resume and contact pages |
| `CONTACT_EMAIL` | `AUTHOR_EMAIL` | Where the contact form sends mail |
| `RESEND_API_KEY` | -- | Resend key; without it the contact form cannot send |
| `TURNSTILE_SITE_KEY`, `TURNSTILE_SECRET_KEY` | -- | Cloudflare Turnstile on the contact form; without them only the honeypot protects it |
| `ADMIN_SECRET` | -- | Secret for `/admin/reload` and `/admin/analytics`; without it both are off |
| `REDIS_URL` | -- | Redis for view counters and session state |
| `CLOUDINARY_BASE_URL` | the project's Cloudinary folder | Base URL for covers and figure PNGs |
| `RUST_LOG` | -- | Log filter, for example `nebula=info,tower_http=info` |

Analytics settings (`ANALYTICS_REPORT_SCHEDULE`, `ANALYTICS_REPORT_EMAIL`,
`ANALYTICS_RETENTION_DAYS`, `ANALYTICS_EXCLUDE_IPS`, `GEOIP_URL`) are described
in [ANALYTICS.md](ANALYTICS.md#configuration).

`HOST` and `SMTP_*` are read into `Config` but nothing uses them.

## Checks

CI runs the same commands on every push and pull request:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo build --release
cargo test
```

The pre-commit hook in `.hooks/` runs `cargo fmt --all` and then `git add -u`,
which stages every modified tracked file, not only the formatted ones. Enable it
with:

```bash
git config core.hooksPath .hooks
```

## Figures and dev.to Export

Both scripts render with headless Chrome (`CHROME` overrides the path).

```bash
# Render every figure of every post, dark and light, into a directory; upload nothing
node scripts/publish-figures.mjs --dry-run /tmp/figs

# Build a dev.to version of a post
node scripts/export-devto.mjs <slug> [--theme light|dark]
```

How figures are written is in [CONTENT_GUIDE.md](CONTENT_GUIDE.md#inline-figures-in-feeds-and-devto)
and [VISUAL_STYLE.md](VISUAL_STYLE.md).

## CI Workflows

| Workflow | Trigger | What it does |
|----------|---------|--------------|
| `build.yml` | push, pull request | The checks above |
| `content.yml` | push to `main` touching `content/**`, manual | Renders and uploads new figures, copies `content/` to the server, calls `/admin/reload` |
| `release.yml` | tag `v*.*.*` | Checks, Docker image to GHCR, covers and figures to Cloudinary, GitHub release from the changelog, deploy |

Secrets: `VPS_HOST`, `VPS_USER`, `VPS_SSH_KEY`, `ADMIN_SECRET`,
`CLOUDINARY_CLOUD_NAME`, `CLOUDINARY_API_KEY`, `CLOUDINARY_API_SECRET`.

A new cover reaches Cloudinary only through `release.yml`. A post with a new
cover pushed through `content.yml` alone shows a broken cover until the next
release.

## Releasing

1. Set the version in `Cargo.toml` and run `cargo check` to update `Cargo.lock`.
2. In `CHANGELOG.md`, add `## [X.Y.Z] - YYYY-MM-DD` right under
   `## [Unreleased]`, so the unreleased entries move into the new version, and
   update the compare links at the bottom.
3. Commit as `chore: release vX.Y.Z`, push `main`, then tag and push the tag:

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

`release.yml` takes the GitHub release notes from the `[X.Y.Z]` section of the
changelog.

## Deployment

The server keeps the stack in `/opt/nebula`: `docker-compose.prod.yml`, `.env`
with secrets, and `content/`, mounted read-only into the app. `release.yml`
copies `content/` and runs:

```bash
cd /opt/nebula
docker compose -f docker-compose.prod.yml pull
docker compose -f docker-compose.prod.yml up -d
```

The deploy does not copy `docker-compose.prod.yml`. A change to that file has to
be copied to the server by hand. The nginx site config and a server bootstrap
script are in `deploy/`.
