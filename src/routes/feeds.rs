use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use rss::{ChannelBuilder, ItemBuilder};

use crate::state::AppState;

pub async fn rss(State(state): State<AppState>) -> Response {
    let content = state.content.read().await;

    let items: Vec<_> = content
        .published_posts()
        .into_iter()
        .take(20)
        .map(|post| {
            ItemBuilder::default()
                .title(Some(post.metadata.title.clone()))
                .link(Some(format!(
                    "{}/blog/{}",
                    state.config.site_url, post.metadata.slug
                )))
                .description(post.metadata.description.clone())
                .pub_date(Some(post.metadata.date.to_rfc2822()))
                .content(Some(post.content_html.clone()))
                .build()
        })
        .collect();

    let channel = ChannelBuilder::default()
        .title(&state.config.site_title)
        .link(&state.config.site_url)
        .description(&state.config.site_description)
        .items(items)
        .build();

    (
        [(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")],
        channel.to_string(),
    )
        .into_response()
}

pub async fn sitemap(State(state): State<AppState>) -> Response {
    let content = state.content.read().await;
    let base_url = &state.config.site_url;

    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push_str(r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#);

    // Static pages
    for path in &["", "/blog", "/projects", "/resume", "/contact"] {
        xml.push_str(&format!(
            "<url><loc>{}{}</loc><changefreq>weekly</changefreq></url>",
            base_url, path
        ));
    }

    // Blog posts
    for post in content.published_posts() {
        xml.push_str(&format!(
            "<url><loc>{}/blog/{}</loc><lastmod>{}</lastmod><changefreq>monthly</changefreq></url>",
            base_url,
            post.metadata.slug,
            post.metadata
                .updated
                .unwrap_or(post.metadata.date)
                .format("%Y-%m-%d")
        ));
    }

    // Projects
    for project in content.all_projects() {
        xml.push_str(&format!(
            "<url><loc>{}/projects/{}</loc><lastmod>{}</lastmod><changefreq>monthly</changefreq></url>",
            base_url,
            project.metadata.slug,
            project.metadata.updated.unwrap_or(project.metadata.date).format("%Y-%m-%d")
        ));
    }

    xml.push_str("</urlset>");

    (
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        xml,
    )
        .into_response()
}
