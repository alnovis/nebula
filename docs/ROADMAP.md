# Nebula - Roadmap

Development roadmap and feature planning for the Nebula project.

## Vision

Build a fast, maintainable personal website that serves as both a professional presence and a learning platform for Rust web development.

## Phases

### Phase 1 - Core Features (v0.1.0) [Complete]

Foundation of the website with essential functionality.

| Feature | Status | Description |
|---------|--------|-------------|
| Axum routing | Done | Basic HTTP routing and handlers |
| Askama templates | Done | Compile-time checked HTML templates |
| Markdown parsing | Done | pulldown-cmark for content |
| Syntax highlighting | Done | syntect for code blocks |
| Blog listing | Done | List and single post views |
| Project showcase | Done | List and detail views |
| RSS feed | Done | `/rss.xml` endpoint |
| Sitemap | Done | `/sitemap.xml` for SEO |
| Health check | Done | `/health` endpoint |
| Resume page | Done | `/resume` with skills and experience |
| Contact form | Done | `/contact` with email integration |
| Docker build | Done | Multi-stage Alpine image |
| Traefik setup | Done | HTTPS with Let's Encrypt |
| CI/CD pipeline | Done | GitHub Actions workflow |

**Release criteria:** Site is deployable and serves static content.

---

### Phase 2 - Content Management (v0.2.0)

Improve content authoring and management workflow.

| Feature | Status | Priority | Description |
|---------|--------|----------|-------------|
| Hot reload | Planned | High | File watcher for development |
| Draft preview | Planned | High | Preview drafts with token |
| Admin UI | Planned | Medium | Web interface for editing |
| Image optimization | Planned | Medium | Automatic resize and WebP |
| Table of contents | Planned | Low | Auto-generated from headings |
| Related posts | Planned | Low | Show similar content |

**Key tasks:**
- [ ] Implement `notify` crate for file watching
- [ ] Add `/preview/:token/:slug` route for drafts
- [ ] Create simple admin authentication (session-based)
- [ ] Build markdown editor with live preview
- [ ] Integrate image processing pipeline

---

### Phase 3 - Engagement (v0.3.0)

Add features for reader interaction and analytics.

| Feature | Status | Priority | Description |
|---------|--------|----------|-------------|
| View counter | Planned | High | Track page views in PostgreSQL |
| Newsletter | Planned | High | Email subscription with double opt-in |
| Search | Planned | Medium | Full-text search across content |
| Comments | Planned | Medium | Via GitHub Issues API |
| Share buttons | Planned | Low | Social media sharing |
| Reading progress | Planned | Low | Progress bar for long posts |

**Key tasks:**
- [ ] Add page view tracking middleware
- [ ] Implement subscriber management with confirmation emails
- [ ] Build search index (tantivy or PostgreSQL FTS)
- [ ] Create GitHub Issues integration for comments
- [ ] Add HTMX-powered interactions

---

### Phase 4 - Performance (v0.4.0)

Optimize for speed and SEO.

| Feature | Status | Priority | Description |
|---------|--------|----------|-------------|
| Cache headers | Planned | High | Proper caching for static assets |
| HTML minification | Planned | Medium | Reduce payload size |
| Preload hints | Planned | Medium | Resource hints for critical assets |
| Lazy loading | Planned | Medium | Images below the fold |
| Edge caching | Planned | Low | Cloudflare or similar CDN |
| Core Web Vitals | Planned | Low | Optimize LCP, FID, CLS |

**Key tasks:**
- [ ] Configure `tower-http` caching layer
- [ ] Add HTML minification middleware
- [ ] Implement `<link rel="preload">` for fonts/CSS
- [ ] Add `loading="lazy"` to images
- [ ] Set up Cloudflare with appropriate cache rules

---

### Phase 5 - Advanced Features (v1.0.0)

Polish and additional capabilities.

| Feature | Status | Priority | Description |
|---------|--------|----------|-------------|
| Multi-language | Planned | Medium | i18n support (EN/RU) |
| Dark/light toggle | Planned | Low | Theme switching |
| Code playground | Planned | Low | Interactive Rust examples |
| API endpoints | Planned | Low | JSON API for content |
| Webhooks | Planned | Low | Notify on new content |

---

## Backlog

Ideas for future consideration (not scheduled):

- **Webmentions** - IndieWeb support
- **ActivityPub** - Fediverse integration
- **Podcast feed** - Audio content support
- **Guestbook** - Retro-style visitor messages
- **Analytics dashboard** - Self-hosted stats
- **A/B testing** - Experiment framework
- **Backup automation** - Scheduled content backups

---

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| v0.1.0 | 2025-01-25 | Initial release with core features, Resume, Contact form |

---

## Contributing

Feature requests and suggestions welcome via GitHub Issues.

When proposing new features, please include:
1. Use case / problem being solved
2. Proposed solution
3. Alternatives considered
4. Impact on existing functionality
