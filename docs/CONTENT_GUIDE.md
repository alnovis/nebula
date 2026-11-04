# Content Guide

This guide covers how to add content to the blog, create projects, prepare cover images, and understand the deployment pipeline.

## Table of Contents

- [Adding a Blog Post](#adding-a-blog-post)
- [Adding a Project](#adding-a-project)
- [Cover Images](#cover-images)
- [Deployment Pipeline](#deployment-pipeline)
- [Local Development](#local-development)

---

## Adding a Blog Post

### 1. Create the Markdown File

Create a new `.md` file in `content/blog/`:

```bash
touch content/blog/my-new-post.md
```

### 2. Add Frontmatter

Every post requires YAML frontmatter:

```yaml
---
title: "Post Title"
slug: "post-slug"
description: "Short description for SEO and social cards"
date: "2026-02-05T12:00:00Z"
tags: ["rust", "web", "tutorial"]
draft: false
cover_image: "my-post-cover.svg"
---
```

| Field | Required | Description |
|-------|----------|-------------|
| `title` | Yes | Post title |
| `slug` | Yes | URL slug (must match filename without `.md`) |
| `description` | Yes | SEO description, shown in cards |
| `date` | Yes | ISO 8601 format |
| `tags` | Yes | Array of tags |
| `draft` | No | Set `true` to hide from listing (default: `false`) |
| `cover_image` | No | Cover filename in `static/images/` (SVG), or a full URL |

### 3. Write Content

Use standard Markdown with:

- **Code blocks** with syntax highlighting (specify language)
- **Mermaid diagrams** in fenced blocks (lazy-loaded)
- **Images** via Cloudinary URLs

```markdown
## Introduction

Some text here...

```rust
fn main() {
    println!("Hello, world!");
}
```

```mermaid
flowchart LR
    A --> B --> C
```
```

### 4. Preview Locally

```bash
cargo run
# Visit http://localhost:3000/blog/post-slug
```

### 5. Publish

Commit and push to `main`. The content will be synced automatically.

---

## Adding a Project

### 1. Create the Markdown File

```bash
touch content/projects/my-project.md
```

### 2. Add Frontmatter

```yaml
---
title: "Project Name"
slug: "project-slug"
description: "Project description"
date: "2026-02-05T12:00:00Z"
tags: ["rust", "cli", "tool"]
status: "active"
github_url: "https://github.com/user/repo"
featured: true
cover_image: "project-cover.svg"
---
```

| Field | Required | Description |
|-------|----------|-------------|
| `title` | Yes | Project name |
| `slug` | Yes | URL slug |
| `description` | Yes | Project description |
| `date` | Yes | Creation/publish date |
| `tags` | Yes | Technology tags |
| `status` | Yes | `active`, `completed`, `archived`, or `planned` |
| `github_url` | No | GitHub repository URL |
| `featured` | No | Show on home page (default: `false`) |
| `cover_image` | No | Cover filename in `static/images/` (SVG), or a full URL |

---

## Cover Images

Cover images are **hand-crafted SVG** files (not stock photos or raster exports).
They live in `static/images/`, are shown on post/project pages and social cards,
and are served from Cloudinary in production.

Colours, typography, layout and a starting template for covers and in-article
figures are in [VISUAL_STYLE.md](VISUAL_STYLE.md).

### Format

| Property | Value |
|----------|-------|
| Format | SVG (hand-crafted, inline vector) |
| Blog `viewBox` | `75 50 1050 580` (cropped for tighter alignment with content) |
| Project `viewBox` | `0 0 1200 630` (2:1 OG ratio) |
| Rendered aspect-ratio | `105 / 58` for blog covers (set in CSS) |
| Location | `static/images/{name}.svg` |

WebP is still accepted by the pipeline for backwards compatibility, but new
covers are authored as SVG.

### Preparation Steps

#### 1. Create the SVG

Author the cover as an SVG by hand. For blog covers use `viewBox="75 50 1050 580"`;
for project covers use the full `viewBox="0 0 1200 630"`. Blog covers render at a
`105 / 58` aspect ratio, so keep the important content away from the cropped edges.

#### 2. Naming

Cover filenames are **thematic**, not derived from the slug. The `-cover` suffix
is a convention; what matters is that the filename matches the `cover_image` field
in frontmatter. A reworked cover gets a version suffix (`-v2`, `-v3`) instead of
replacing the old file, so cached copies cannot show the old image
(see [VISUAL_STYLE.md](VISUAL_STYLE.md#publishing)).

```
static/images/
├── aggregate-function-cover.svg    # slug: aggregate-is-a-function
├── compiler-ideas-cover-v2.svg     # slug: compiler-ideas-for-code-generation
├── cs-in-ai-era-cover-v2.svg       # slug: do-you-need-cs-in-ai-era
├── design-recovery-cover-v2.svg    # slug: redesigning-my-blog
├── rust-web-cover-v2.svg           # slug: hello-world
└── proto-wrapper-cover-v2.svg      # slug: proto-wrapper-plugin
```

#### 3. Update Frontmatter

Add the filename to your post (not a full URL):

```yaml
cover_image: "my-post-cover.svg"
```

The system resolves the URL automatically (`resolve_cover_url` in `src/config.rs`):

- **Production**: extension is stripped and the image is served by Cloudinary
  public_id -> `https://res.cloudinary.com/ddkzhz9b4/image/upload/nebula/my-post-cover`
- **Development**: `/static/images/my-post-cover.svg` (served from `static/images/`)
- A full `http(s)://` URL is used as-is.

This means cover images work locally without uploading to Cloudinary first.

#### 4. Upload Happens Automatically

Images in `static/images/` are uploaded to Cloudinary during the release workflow
(when you push a `v*.*.*` tag). Both `*.svg` and `*.webp` are uploaded.

For manual upload (requires Cloudinary credentials):

```bash
export CLOUDINARY_API_KEY="your-key"
export CLOUDINARY_API_SECRET="your-secret"
./scripts/upload-images.sh
```

Note: `scripts/upload-images.sh` currently uploads only `*.webp`. SVG covers are
uploaded by the release workflow (`release.yml`), so rely on a tagged release (or
extend the script) to publish new SVG covers.

---

## Deployment Pipeline

### Overview

```mermaid
flowchart TB
    subgraph triggers["Triggers"]
        T1["Push to main\n(content/* only)"]
        T2["Push tag\n(v*.*.*)"]
    end

    subgraph content_flow["Content Update Flow"]
        direction TB
        C1["Sync content to VPS"]
        C2["Reload app content"]
    end

    subgraph release_flow["Release Flow"]
        direction TB
        R1["Validate\n(fmt, clippy, test)"]
        R2["Build Docker image"]
        R3["Upload images to\nCloudinary"]
        R4["Create GitHub Release"]
        R5["Deploy to VPS"]
        R6["Reload content"]
        R1 --> R2
        R1 --> R3
        R2 --> R4
        R3 --> R4
        R4 --> R5
        R5 --> R6
    end

    T1 --> content_flow
    T2 --> release_flow
```

### Workflows

#### 1. `build.yml` — Continuous Integration

**Trigger:** Push or PR to `main`

**Steps:**
1. Check formatting (`cargo fmt --check`)
2. Run Clippy (`cargo clippy -- -D warnings`)
3. Build (`cargo build --release`)
4. Run tests (`cargo test`)

#### 2. `content.yml` — Content Updates

**Trigger:** Push to `main` that changes `content/**`

**Steps:**
1. Sync `content/` directory to VPS via SCP
2. Call `/admin/reload` endpoint to hot-reload content

This allows publishing new posts without rebuilding the Docker image.

#### 3. `release.yml` — Full Release

**Trigger:** Push a version tag (`v*.*.*`)

**Steps:**
1. **Validate** — format, lint, build, test
2. **Build Docker** — multi-stage build, push to `ghcr.io`
3. **Upload Media** — upload `static/images/*.svg` and `*.webp` to Cloudinary
4. **Create Release** — GitHub release with changelog
5. **Deploy** — pull image on VPS, restart container
6. **Reload** — trigger content reload, health check

### Publishing Workflow

#### Content-Only Update (No Code Changes)

```bash
# 1. Add/edit content
vim content/blog/my-post.md

# 2. Add cover image (if any) -- author an SVG in static/images/
vim static/images/my-post-cover.svg

# 3. Commit and push
git add content/ static/images/
git commit -m "Add new blog post"
git push origin main
```

The `content.yml` workflow syncs content automatically. Images will be uploaded on next release.

#### Full Release (Code + Content)

```bash
# 1. Prepare changes
vim src/routes/blog.rs
vim content/blog/my-post.md

# 2. Run release script
./scripts/release.sh patch  # or minor/major/0.3.0

# 3. Edit CHANGELOG.md with actual changes

# 4. Amend commit and push
git add CHANGELOG.md
git commit --amend --no-edit
git push origin main --tags
```

### Environment Secrets Required

| Secret | Description |
|--------|-------------|
| `VPS_HOST` | VPS IP or hostname |
| `VPS_USER` | SSH username |
| `VPS_SSH_KEY` | SSH private key |
| `ADMIN_SECRET` | Secret for `/admin/reload` endpoint |
| `CLOUDINARY_CLOUD_NAME` | Cloudinary cloud name |
| `CLOUDINARY_API_KEY` | Cloudinary API key |
| `CLOUDINARY_API_SECRET` | Cloudinary API secret |

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ENVIRONMENT` | `development` | Set to `production` for Cloudinary URLs |
| `CLOUDINARY_BASE_URL` | `https://res.cloudinary.com/ddkzhz9b4/image/upload/nebula` | Base URL for images |

---

## Local Development

### Prerequisites

- Rust 1.75+
- PostgreSQL 16+
- Docker (for database)

### Setup

```bash
# Start PostgreSQL
docker compose up -d

# Copy environment
cp .env.example .env

# Run
cargo run
```

### Hot Reload Content

Content is loaded at startup. To reload without restart:

```bash
curl -X POST "http://localhost:3000/admin/reload?secret=your-secret"
```

### Testing Posts

Set `draft: true` in frontmatter to hide from listing while testing.

```yaml
draft: true
```

The post is still accessible directly at `/blog/{slug}` for preview.

---

## Quick Reference

### File Locations

| Content Type | Location |
|--------------|----------|
| Blog posts | `content/blog/*.md` |
| Projects | `content/projects/*.md` |
| Cover images | `static/images/*-cover.svg` |

### URL Patterns

| Type | URL |
|------|-----|
| Blog post | `/blog/{slug}` |
| Project | `/projects/{slug}` |
| Cover image (prod) | `https://res.cloudinary.com/ddkzhz9b4/image/upload/nebula/{name}-cover` |
| Cover image (dev) | `/static/images/{name}-cover.svg` |

### Commands

```bash
# Local dev
cargo run

# Manual content reload
curl -X POST "http://localhost:3000/admin/reload?secret=$ADMIN_SECRET"

# Create release
./scripts/release.sh patch
```
