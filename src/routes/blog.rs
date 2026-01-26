use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Html;

use crate::state::AppState;
use crate::VERSION;

#[derive(Template)]
#[template(path = "blog/list.html")]
struct BlogListTemplate<'a> {
    title: &'a str,
    nav_path: &'a str,
    version: &'a str,
    canonical_url: String,
    og_type: &'a str,
    og_image: Option<&'a str>,
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
    og_image: Option<&'a str>,
    description: Option<&'a str>,
    date: String,
    reading_time: u32,
    tags: &'a [String],
    content: &'a str,
    cover_image: Option<&'a str>,
}

struct PostItem<'a> {
    title: &'a str,
    slug: &'a str,
    description: Option<&'a str>,
    date: String,
    reading_time: u32,
    tags: &'a [String],
    cover_image: Option<&'a str>,
}

pub async fn list(State(state): State<AppState>) -> Html<String> {
    let content = state.content.read().await;

    let posts: Vec<_> = content
        .published_posts()
        .into_iter()
        .map(|p| PostItem {
            title: &p.metadata.title,
            slug: &p.metadata.slug,
            description: p.metadata.description.as_deref(),
            date: p.metadata.date.format("%Y-%m-%d").to_string(),
            reading_time: p.reading_time_minutes,
            tags: &p.metadata.tags,
            cover_image: p.metadata.cover_image.as_deref(),
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
) -> Result<Html<String>, StatusCode> {
    let content = state.content.read().await;

    let post = content.posts.get(&slug).ok_or(StatusCode::NOT_FOUND)?;

    if post.metadata.draft {
        return Err(StatusCode::NOT_FOUND);
    }

    let cover_image = post.metadata.cover_image.as_deref();

    let template = BlogPostTemplate {
        title: &post.metadata.title,
        nav_path: "/blog",
        version: VERSION,
        canonical_url: format!("{}/blog/{}", state.config.site_url, slug),
        og_type: "article",
        og_image: cover_image,
        description: post.metadata.description.as_deref(),
        date: post.metadata.date.format("%Y-%m-%d").to_string(),
        reading_time: post.reading_time_minutes,
        tags: &post.metadata.tags,
        content: &post.content_html,
        cover_image,
    };

    Ok(Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    ))
}
