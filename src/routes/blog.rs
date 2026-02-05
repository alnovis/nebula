use askama::Template;
use axum::extract::{ConnectInfo, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Html;
use std::net::SocketAddr;

use crate::state::AppState;
use crate::views::{self, ContentType, ViewsService};
use crate::VERSION;

#[derive(Template)]
#[template(path = "blog/list.html")]
struct BlogListTemplate<'a> {
    title: &'a str,
    nav_path: &'a str,
    version: &'a str,
    canonical_url: String,
    og_type: &'a str,
    og_image: Option<String>,
    posts: Vec<PostItem<'a>>,
}

#[derive(Template)]
#[template(path = "blog/post.html")]
struct BlogPostTemplate<'a> {
    title: &'a str,
    nav_path: &'a str,
    version: &'a str,
    canonical_url: String,
    og_type: &'a str,
    og_image: Option<String>,
    description: Option<&'a str>,
    date: String,
    reading_time: u32,
    tags: &'a [String],
    content: &'a str,
    cover_image: Option<String>,
    views_count: Option<String>,
}

struct PostItem<'a> {
    title: &'a str,
    slug: &'a str,
    description: Option<&'a str>,
    date: String,
    reading_time: u32,
    tags: &'a [String],
    cover_image: Option<String>,
    views_count: Option<String>,
}

pub async fn list(State(state): State<AppState>) -> Html<String> {
    let content = state.content.read().await;
    let published = content.published_posts();

    // Batch fetch view counts if Redis is available
    let view_counts: Vec<Option<String>> = if let Some(ref redis) = state.redis {
        let service = ViewsService::new(redis.clone());
        let slugs: Vec<&str> = published.iter().map(|p| p.metadata.slug.as_str()).collect();
        match service.get_counts(ContentType::Post, &slugs).await {
            Ok(counts) => counts
                .into_iter()
                .map(|c| Some(views::format_count(c)))
                .collect(),
            Err(_) => vec![None; published.len()],
        }
    } else {
        vec![None; published.len()]
    };

    let posts: Vec<_> = published
        .into_iter()
        .zip(view_counts)
        .map(|(p, views_count)| {
            let cover_image = p
                .metadata
                .cover_image
                .as_ref()
                .map(|c| state.config.resolve_cover_url(c));
            PostItem {
                title: &p.metadata.title,
                slug: &p.metadata.slug,
                description: p.metadata.description.as_deref(),
                date: p.metadata.date.format("%Y-%m-%d").to_string(),
                reading_time: p.reading_time_minutes,
                tags: &p.metadata.tags,
                cover_image,
                views_count,
            }
        })
        .collect();

    let template = BlogListTemplate {
        title: "Blog",
        nav_path: "/blog",
        version: VERSION,
        canonical_url: format!("{}/blog", state.config.site_url),
        og_type: "website",
        og_image: None,
        posts,
    };

    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
}

pub async fn show(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    headers: HeaderMap,
    peer: Option<ConnectInfo<SocketAddr>>,
) -> Result<Html<String>, StatusCode> {
    let content = state.content.read().await;

    let post = content.posts.get(&slug).ok_or(StatusCode::NOT_FOUND)?;

    if post.metadata.draft {
        return Err(StatusCode::NOT_FOUND);
    }

    // Get views count and record view if Redis is available
    let views_count = if let Some(ref redis) = state.redis {
        let service = ViewsService::new(redis.clone());

        // Get current count first (for display)
        let count = service
            .get_count(ContentType::Post, &slug)
            .await
            .unwrap_or(0);

        // Record view in background (fire and forget)
        let ip = views::extract_client_ip(&headers, peer.map(|p| p.0));
        let ua = views::extract_user_agent(&headers);
        if let Some(ip) = ip {
            let service = ViewsService::new(redis.clone());
            let slug_owned = slug.clone();
            tokio::spawn(async move {
                let _ = service
                    .record_view(ContentType::Post, &slug_owned, &ip, ua.as_deref())
                    .await;
            });
        }

        Some(views::format_count(count))
    } else {
        None
    };

    let cover_image = post
        .metadata
        .cover_image
        .as_ref()
        .map(|c| state.config.resolve_cover_url(c));

    let template = BlogPostTemplate {
        title: &post.metadata.title,
        nav_path: "/blog",
        version: VERSION,
        canonical_url: format!("{}/blog/{}", state.config.site_url, slug),
        og_type: "article",
        og_image: cover_image.clone(),
        description: post.metadata.description.as_deref(),
        date: post.metadata.date.format("%Y-%m-%d").to_string(),
        reading_time: post.reading_time_minutes,
        tags: &post.metadata.tags,
        content: &post.content_html,
        cover_image,
        views_count,
    };

    Ok(Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    ))
}
