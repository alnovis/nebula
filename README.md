# Nebula

Personal website and blog engine built with Rust, Axum, and HTMX.

Live: [alnovis.io](https://alnovis.io)

## Features

### Core
- **Rust + Axum** — fast, async web framework
- **Askama templates** — type-safe, compile-time templates with partials
- **HTMX** — interactivity without heavy JavaScript
- **PostgreSQL** — contact form submissions storage
- **Redis** — views counter with unique visitor tracking (optional)

### Content
- **Markdown** with YAML frontmatter
- **Syntax highlighting** via Syntect (One Dark theme)
- **Mermaid diagrams** — lazy-loaded only when needed
- **Cover images** for blog posts and projects
- **Reading time** estimation
- **Views counter** — unique visitor tracking with bot filtering
- **RSS feed** and **sitemap** generation

### Performance
- **Critical CSS** inlined in `<head>` for fast first paint
- **Deferred CSS** loading (`media="print" onload`)
- **CDN fallback** for external scripts (jsdelivr → cdnjs → unpkg)
- **Cloudinary** for image hosting and optimization
- **Gzip compression** via tower-http

### SEO & Social
- **Open Graph** meta tags
- **Twitter Cards** support
- **Canonical URLs**
- **robots.txt** and **sitemap.xml**
- **Favicon** in multiple sizes (16, 32, 48, 180, 192px)

### Security
- **Cloudflare Turnstile** captcha on contact form
- **Resend** for email delivery

### Deployment
- **Docker** multi-stage build
- **GitHub Actions** CI/CD
- **Traefik** reverse proxy with auto SSL
- **Admin endpoint** for hot content reload

## Quick Start

### Prerequisites

- Rust 1.75+
- PostgreSQL 16+
- Docker (for deployment)

### Development

```bash
# Start PostgreSQL
docker compose up -d

# Copy environment file
cp .env.example .env

# Run the application
cargo run
```

Visit http://localhost:3000

### Environment Variables

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | PostgreSQL connection string |
| `SITE_URL` | Public site URL |
| `RESEND_API_KEY` | Resend API key for emails |
| `TURNSTILE_SECRET_KEY` | Cloudflare Turnstile secret |
| `ADMIN_SECRET` | Secret for admin endpoints |
| `REDIS_URL` | Redis connection string (optional, for views counter) |

## Project Structure

```
nebula/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # App router setup
│   ├── config.rs            # Configuration
│   ├── state.rs             # Shared state
│   ├── email.rs             # Resend integration
│   ├── turnstile.rs         # Captcha verification
│   ├── views.rs             # Views counter (Redis)
│   ├── content/             # Markdown parsing
│   │   ├── mod.rs           # ContentStore
│   │   └── markdown.rs      # MD → HTML conversion
│   ├── models/              # Data models
│   │   ├── post.rs          # Blog post
│   │   └── project.rs       # Project
│   └── routes/              # HTTP handlers
│       ├── pages.rs         # Home page
│       ├── blog.rs          # Blog list/post
│       ├── projects.rs      # Projects list/show
│       ├── resume.rs        # Resume/CV
│       ├── contact.rs       # Contact form
│       ├── feeds.rs         # RSS, sitemap, robots
│       ├── admin.rs         # Content reload
│       └── health.rs        # Health check
├── templates/
│   ├── base.html            # Base layout
│   ├── index.html           # Home page
│   ├── blog/                # Blog templates
│   ├── projects/            # Projects templates
│   ├── partials/
│   │   ├── critical-css.html  # Inline critical CSS
│   │   └── scripts.html       # JS with CDN fallback
│   └── ...
├── static/
│   ├── css/style.css        # Full stylesheet
│   ├── js/main.js           # External JS fallback
│   ├── favicon*.png         # Favicons
│   └── images/              # Local images
├── content/
│   ├── blog/                # Blog posts (*.md)
│   └── projects/            # Projects (*.md)
├── migrations/              # SQL migrations
├── scripts/
│   └── upload-images.sh     # Cloudinary upload
├── .hooks/
│   └── pre-commit           # cargo fmt hook
└── .github/
    └── workflows/
        └── deploy.yml       # CI/CD pipeline
```

## Content Format

### Blog Post

```markdown
---
title: "Post Title"
slug: "post-slug"
description: "Short description for SEO"
date: "2025-01-27T10:00:00Z"
tags: ["rust", "web"]
cover_image: "https://res.cloudinary.com/xxx/image/upload/cover.webp"
---

Content here...
```

### Project

```markdown
---
title: "Project Name"
slug: "project-slug"
description: "Project description"
date: "2025-01-27T10:00:00Z"
tags: ["rust", "cli"]
status: "active"
github_url: "https://github.com/user/repo"
featured: true
cover_image: "https://res.cloudinary.com/xxx/image/upload/cover.webp"
---

Content here...
```

## Deployment

### GitHub Actions

Push a tag to trigger deployment:

```bash
git tag v0.2.19
git push origin v0.2.19
```

The workflow will:
1. Upload images to Cloudinary
2. Build Docker image
3. Push to GitHub Container Registry
4. Deploy to VPS via SSH
5. Reload content

### Manual Deployment

```bash
docker compose -f docker-compose.prod.yml pull
docker compose -f docker-compose.prod.yml up -d
```

### Hot Content Reload

```bash
curl -X POST "https://alnovis.io/admin/reload?secret=$ADMIN_SECRET"
```

## Development

### Git Hooks

```bash
# Enable pre-commit hook (cargo fmt)
git config core.hooksPath .hooks
```

### Release Process

```bash
# Generate changelog from commits
./scripts/changelog-gen.sh

# Create release (interactive)
./scripts/release.sh 0.3.0

# Or bump version automatically
./scripts/release.sh patch  # 0.2.19 -> 0.2.20
./scripts/release.sh minor  # 0.2.19 -> 0.3.0
./scripts/release.sh major  # 0.2.19 -> 1.0.0

# Preview without changes
./scripts/release.sh 0.3.0 --dry-run
```

The release script will:
1. Update version in `Cargo.toml`
2. Add section to `CHANGELOG.md`
3. Update `RELEASE_NOTES.md`
4. Create commit and tag

After running, edit the changelog with actual changes, then push:

```bash
git push && git push --tags
```

### Adding Content

1. Create `.md` file in `content/blog/` or `content/projects/`
2. Add frontmatter with required fields
3. Restart server or call reload endpoint

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust 1.75+ |
| Web Framework | Axum 0.7 |
| Templates | Askama 0.12 |
| Database | PostgreSQL 16 + SQLx |
| Markdown | pulldown-cmark + Syntect |
| Diagrams | Mermaid (lazy-loaded) |
| Interactivity | HTMX 1.9 |
| Images | Cloudinary CDN |
| Email | Resend API |
| Captcha | Cloudflare Turnstile |
| Deployment | Docker + GitHub Actions |
| Reverse Proxy | Traefik |
| Analytics | Cloudflare Web Analytics |

## License

MIT
