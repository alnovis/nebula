# Nebula - Technical Documentation

Personal website and blog engine for alnovis.io.

## Overview

Nebula is a lightweight, fast personal website engine built with Rust. It serves as both a production website and a learning project for Rust web development.

### Goals

1. **Professional presence** - Blog and project showcase
2. **Learning Rust** - Practical application of Rust concepts
3. **Simple deployment** - Single Docker container
4. **Performance** - Sub-millisecond response times

## Architecture

```
                                    ┌─────────────────────────────────────────┐
                                    │              VPS (Germany)              │
                                    │                                         │
    ┌──────────┐     HTTPS          │  ┌─────────┐      ┌──────────────────┐  │
    │  Users   │◄──────────────────►│  │ Traefik │◄────►│     Nebula       │  │
    └──────────┘     :443           │  │ (proxy) │      │   (Rust/Axum)    │  │
                                    │  └─────────┘      └────────┬─────────┘  │
                                    │       │                    │            │
                                    │       │           ┌────────▼─────────┐  │
                                    │       │           │   PostgreSQL     │  │
                                    │       │           │   (analytics)    │  │
                                    │       │           └──────────────────┘  │
                                    │       │                                 │
                                    │  Let's Encrypt                          │
                                    │  (auto-certificates)                    │
                                    └─────────────────────────────────────────┘

    ┌──────────────────────────────────────────────────────────────────────────┐
    │                          Content Pipeline                                 │
    │                                                                          │
    │   ┌──────────┐      ┌──────────┐      ┌──────────┐      ┌──────────┐    │
    │   │ Markdown │─────►│  Git     │─────►│ GitHub   │─────►│ Docker   │    │
    │   │  Files   │ push │  Repo    │ push │ Actions  │ push │ Registry │    │
    │   └──────────┘      └──────────┘      └──────────┘      └──────────┘    │
    │                                                               │          │
    │                                                               ▼          │
    │                                                         Watchtower       │
    │                                                         (auto-update)    │
    └──────────────────────────────────────────────────────────────────────────┘
```

## Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Language | Rust | Memory safety, performance |
| Web Framework | Axum | Async, ergonomic routing |
| Templates | Askama | Compile-time checked HTML |
| Interactivity | HTMX | Server-side updates without JS |
| Database | PostgreSQL | Analytics, future features |
| Markdown | pulldown-cmark | CommonMark parsing |
| Syntax Highlighting | syntect | Code block highlighting |
| Reverse Proxy | Traefik | HTTPS, routing |
| Container | Docker | Deployment packaging |
| CI/CD | GitHub Actions | Build, test, deploy |

## Project Structure

```
nebula/
├── Cargo.toml              # Dependencies and metadata
├── Dockerfile              # Multi-stage Alpine build
├── docker-compose.yml      # Development setup
├── docker-compose.prod.yml # Production with Traefik
├── .env.example            # Environment template
│
├── src/
│   ├── main.rs             # Entry point, logging setup
│   ├── lib.rs              # Router and app setup
│   ├── config.rs           # Environment configuration
│   ├── state.rs            # Shared application state
│   │
│   ├── content/
│   │   ├── mod.rs          # Content store
│   │   └── markdown.rs     # Markdown parsing + syntax highlighting
│   │
│   ├── models/
│   │   ├── mod.rs
│   │   ├── post.rs         # Blog post model
│   │   └── project.rs      # Project model
│   │
│   └── routes/
│       ├── mod.rs
│       ├── pages.rs        # Index, about pages
│       ├── blog.rs         # Blog list and post views
│       ├── projects.rs     # Project list and detail views
│       ├── resume.rs       # Resume/CV page
│       ├── contact.rs      # Contact form with email
│       ├── feeds.rs        # RSS and sitemap
│       └── health.rs       # Health check endpoint
│
├── templates/
│   ├── base.html           # Base layout
│   ├── index.html          # Homepage
│   ├── about.html          # About page
│   ├── resume.html         # Resume/CV
│   ├── contact.html        # Contact form
│   ├── contact_success.html # Contact form success
│   ├── blog/
│   │   ├── list.html       # Blog listing
│   │   └── post.html       # Single post
│   └── projects/
│       ├── list.html       # Project listing
│       └── show.html       # Single project
│
├── static/
│   └── css/
│       └── style.css       # Dark theme styles
│
├── content/
│   ├── blog/               # Markdown blog posts
│   └── projects/           # Markdown project pages
│
├── migrations/
│   └── 001_initial.sql     # Database schema
│
├── .github/
│   └── workflows/
│       └── ci.yml          # CI/CD pipeline
│
└── docs/
    ├── PLAN.md             # This document
    └── ROADMAP.md          # Feature roadmap
```

## Content Format

### Blog Post Frontmatter

```yaml
---
title: "Post Title"
slug: "post-slug"
description: "Brief description for SEO and listings"
date: "2025-01-06T12:00:00Z"
updated: "2025-01-07T12:00:00Z"  # optional
tags: ["rust", "web"]
draft: false
---

Markdown content here...
```

### Project Frontmatter

```yaml
---
title: "Project Name"
slug: "project-slug"
description: "Brief description"
date: "2025-01-06T12:00:00Z"
tags: ["rust", "cli"]
status: "active"           # active, completed, archived, planned
github_url: "https://github.com/..."
demo_url: "https://..."    # optional
featured: true
---

Markdown content here...
```

## Development

### Prerequisites

- Rust 1.75+ (install via rustup)
- PostgreSQL 16+
- Docker and Docker Compose

### Setup

```bash
# Clone repository
git clone https://github.com/alnovis/nebula.git
cd nebula

# Start PostgreSQL
docker compose up -d

# Configure environment
cp .env.example .env
# Edit .env with your settings

# Run development server
cargo run
```

### Commands

```bash
# Check code
cargo fmt --check
cargo clippy

# Run tests
cargo test

# Build release
cargo build --release

# Build Docker image
docker build -t nebula .
```

## Deployment

### Server Setup

1. **Install Docker and Docker Compose**

```bash
# Ubuntu/Debian
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER
```

2. **Clone repository**

```bash
git clone https://github.com/alnovis/nebula.git
cd nebula
```

3. **Configure environment**

```bash
cp .env.example .env.prod
# Edit .env.prod:
# - DOMAIN=alnovis.io
# - DB_PASSWORD=<secure-password>
# - ACME_EMAIL=dev@alnovis.io
# - TRAEFIK_AUTH=$(htpasswd -nb admin password)
```

4. **Start services**

```bash
docker compose -f docker-compose.prod.yml --env-file .env.prod up -d
```

### DNS Configuration

Point your domain to the server IP:

```
A    alnovis.io      -> <server-ip>
A    www.alnovis.io  -> <server-ip>
```

### Updating

With Watchtower configured, updates are automatic:

1. Push changes to `main` branch
2. GitHub Actions builds and pushes new Docker image
3. Watchtower detects new image and restarts container

Manual update:

```bash
docker compose -f docker-compose.prod.yml pull
docker compose -f docker-compose.prod.yml up -d
```

## Configuration Reference

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `HOST` | No | `0.0.0.0` | Listen address |
| `PORT` | No | `3000` | Listen port |
| `DATABASE_URL` | Yes | - | PostgreSQL connection string |
| `CONTENT_DIR` | No | `./content` | Content directory path |
| `SITE_URL` | No | `http://localhost:3000` | Public site URL |
| `SITE_TITLE` | No | `Nebula` | Site title |
| `SITE_DESCRIPTION` | No | - | Site description |
| `AUTHOR_NAME` | No | `Author` | Author name |
| `AUTHOR_EMAIL` | No | - | Author email |
| `SMTP_HOST` | No | - | SMTP server for contact form |
| `SMTP_PORT` | No | `587` | SMTP server port |
| `SMTP_USER` | No | - | SMTP username |
| `SMTP_PASSWORD` | No | - | SMTP password |
| `CONTACT_EMAIL` | No | `AUTHOR_EMAIL` | Where to send contact messages |
| `TURNSTILE_SITE_KEY` | No | - | Cloudflare Turnstile site key |
| `TURNSTILE_SECRET_KEY` | No | - | Cloudflare Turnstile secret key |
| `RUST_LOG` | No | `nebula=info` | Log level |

### Production Environment

| Variable | Required | Description |
|----------|----------|-------------|
| `DOMAIN` | Yes | Primary domain |
| `DB_PASSWORD` | Yes | PostgreSQL password |
| `ACME_EMAIL` | Yes | Let's Encrypt email |
| `TRAEFIK_AUTH` | Yes | Traefik dashboard auth |

## Monitoring

### Health Check

```bash
curl http://localhost:3000/health
```

Response:
```json
{
  "status": "ok",
  "database": "ok",
  "posts_count": 5,
  "projects_count": 3
}
```

### Logs

```bash
# Application logs
docker logs nebula-app -f

# Traefik logs
docker logs traefik -f

# All services
docker compose -f docker-compose.prod.yml logs -f
```

## Related Documentation

- [ROADMAP.md](ROADMAP.md) - Feature roadmap and development phases

## License

MIT License - see LICENSE file.
