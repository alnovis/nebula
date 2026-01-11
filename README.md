# Nebula

Personal website and blog engine built with Rust, Axum, and HTMX.

[![CI](https://github.com/alnovis/nebula/actions/workflows/ci.yml/badge.svg)](https://github.com/alnovis/nebula/actions/workflows/ci.yml)
[![Docker](https://github.com/alnovis/nebula/actions/workflows/docker.yml/badge.svg)](https://github.com/alnovis/nebula/actions/workflows/docker.yml)

## Features

- Fast server-side rendering with Axum
- HTMX for interactivity without heavy JavaScript
- Markdown content with syntax highlighting
- RSS feed and sitemap generation
- Docker deployment (standalone or with VPN infrastructure)

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

### Production

#### Standalone (with Traefik)

```bash
docker compose -f docker-compose.prod.yml up -d
```

#### With Mirage (VPN Infrastructure)

Nebula integrates with [Mirage](../mirage/) as a camouflage site:

```bash
# In .env
NEBULA_ENABLED=true
NEBULA_DB_PASSWORD=$(openssl rand -base64 32)

# Deploy
cd ../mirage/ansible
ansible-playbook -i inventory.yml playbook.yml
```

See [mirage/README.md](../mirage/README.md) for details.

## Docker Image

```bash
# Pull from GitHub Container Registry
docker pull ghcr.io/alnovis/nebula:latest

# Run
docker run -d \
  -e DATABASE_URL=postgres://user:pass@host:5432/nebula \
  -e SITE_URL=https://example.com \
  -p 3000:3000 \
  ghcr.io/alnovis/nebula:latest
```

## CI/CD

### Workflows

| Workflow | Trigger | Description |
|----------|---------|-------------|
| `ci.yml` | Push/PR to main | Check, lint, test, build |
| `docker.yml` | Push to main, tags | Build & push Docker image |

### Docker Tags

| Tag | Description |
|-----|-------------|
| `latest` | Latest main branch |
| `main` | Main branch |
| `v1.0.0` | Release version |
| `abc1234` | Commit SHA |

## Project Structure

```
nebula/
├── .github/workflows/    # CI/CD
│   ├── ci.yml            # Rust checks & tests
│   └── docker.yml        # Docker build & push
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # App router setup
│   ├── config.rs         # Configuration
│   ├── state.rs          # Shared state
│   ├── content/          # Markdown parsing
│   ├── models/           # Data models
│   └── routes/           # HTTP handlers
├── templates/            # Askama templates
├── static/               # CSS, JS, images
├── content/              # Markdown content
│   ├── blog/
│   └── projects/
├── migrations/           # SQL migrations
├── Dockerfile            # Multi-stage build
└── docker-compose*.yml   # Deployment configs
```

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | - | PostgreSQL connection string |
| `SITE_URL` | No | `http://localhost:3000` | Public site URL |
| `SITE_TITLE` | No | `Nebula` | Site title |
| `PORT` | No | `3000` | HTTP port |
| `RUST_LOG` | No | `nebula=info` | Log level |

## License

MIT
