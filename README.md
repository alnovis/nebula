# Nebula

Personal website and blog engine built with Rust, Axum, and HTMX.

## Features

- Fast server-side rendering with Axum
- HTMX for interactivity without heavy JavaScript
- Markdown content with syntax highlighting
- RSS feed and sitemap generation
- Resume/CV page
- Contact form with email integration
- Docker deployment with Traefik

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

See [docs/PLAN.md](docs/PLAN.md) for detailed deployment instructions.

```bash
# Deploy with Traefik
docker compose -f docker-compose.prod.yml up -d
```

## Project Structure

```
nebula/
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
│   ├── blog/             # Blog posts
│   └── projects/         # Project showcases
├── migrations/           # SQL migrations
└── docs/                 # Documentation
```

## License

MIT
