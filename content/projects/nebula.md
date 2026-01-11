---
title: "Nebula"
slug: "nebula"
description: "Personal website and blog engine built with Rust, Axum, and HTMX"
date: "2025-01-06T12:00:00Z"
tags: ["rust", "axum", "htmx", "web"]
status: "active"
github_url: "https://github.com/alnovis/nebula"
featured: true
---

Nebula is a lightweight, fast personal website engine written in Rust.

## Features

- **Fast**: Sub-millisecond response times
- **Simple**: Markdown files for content, no CMS needed
- **Modern**: HTMX for interactivity without heavy JavaScript
- **Secure**: Built with Rust's memory safety guarantees
- **Deployable**: Single Docker container

## Architecture

The application follows a simple architecture:

1. **Content Store** - Loads markdown files at startup
2. **Axum Router** - Handles HTTP requests
3. **Askama Templates** - Compile-time checked HTML templates
4. **PostgreSQL** - Analytics and future features

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust |
| Framework | Axum |
| Templates | Askama |
| Interactivity | HTMX |
| Database | PostgreSQL |
| Markdown | pulldown-cmark + syntect |

## Deployment

Nebula runs as a single Docker container behind Traefik reverse proxy with automatic HTTPS via Let's Encrypt.
