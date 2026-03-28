use askama::Template;
use axum::extract::State;
use axum::response::Html;

use crate::state::AppState;
use crate::views::{ContentType, ViewsService};
use crate::VERSION;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    title: &'a str,
    nav_path: &'a str,
    version: &'a str,
    canonical_url: String,
    og_type: &'a str,
    og_image: Option<String>,
    recent_posts: Vec<PostSummary<'a>>,
    featured_projects: Vec<ProjectSummary<'a>>,
}

#[allow(dead_code)]
struct PostSummary<'a> {
    title: &'a str,
    slug: &'a str,
    description: Option<&'a str>,
    date: String,
    reading_time: u32,
    cover_image: Option<String>,
    tags: &'a [String],
    views_count: Option<String>,
}

struct ProjectSummary<'a> {
    title: &'a str,
    slug: &'a str,
    description: Option<&'a str>,
    status: &'a str,
    cover_image: Option<String>,
    tags: &'a [String],
}

pub async fn index(State(state): State<AppState>) -> Html<String> {
    let content = state.content.read().await;
    let published: Vec<_> = content.published_posts().into_iter().take(5).collect();

    let view_counts: Vec<Option<String>> = if let Some(ref redis) = state.redis {
        let service = ViewsService::new(redis.clone());
        let slugs: Vec<&str> = published.iter().map(|p| p.metadata.slug.as_str()).collect();
        match service.get_counts(ContentType::Post, &slugs).await {
            Ok(counts) => counts.into_iter().map(|c| Some(c.to_string())).collect(),
            Err(_) => vec![None; published.len()],
        }
    } else {
        vec![None; published.len()]
    };

    let recent_posts: Vec<_> = published
        .into_iter()
        .zip(view_counts)
        .map(|(p, views_count)| {
            let cover_image = p
                .metadata
                .cover_image
                .as_ref()
                .map(|c| state.config.resolve_cover_url(c));
            PostSummary {
                title: &p.metadata.title,
                slug: &p.metadata.slug,
                description: p.metadata.description.as_deref(),
                date: p.metadata.date.format("%Y-%m-%d").to_string(),
                reading_time: p.reading_time_minutes,
                cover_image,
                tags: &p.metadata.tags,
                views_count,
            }
        })
        .collect();

    let featured_projects: Vec<_> = content
        .featured_projects()
        .into_iter()
        .take(3)
        .map(|p| {
            let cover_image = p
                .metadata
                .cover_image
                .as_ref()
                .map(|c| state.config.resolve_cover_url(c));
            ProjectSummary {
                title: &p.metadata.title,
                slug: &p.metadata.slug,
                description: p.metadata.description.as_deref(),
                status: match p.metadata.status {
                    crate::models::project::ProjectStatus::Active => "Active",
                    crate::models::project::ProjectStatus::Completed => "Completed",
                    crate::models::project::ProjectStatus::Archived => "Archived",
                    crate::models::project::ProjectStatus::Planned => "Planned",
                },
                cover_image,
                tags: &p.metadata.tags,
            }
        })
        .collect();

    let template = IndexTemplate {
        title: &state.config.site_title,
        nav_path: "/",
        version: VERSION,
        canonical_url: state.config.site_url.clone(),
        og_type: "website",
        og_image: None,
        recent_posts,
        featured_projects,
    };

    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
}
