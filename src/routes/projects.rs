use askama::Template;
use axum::extract::{ConnectInfo, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Html;
use std::net::SocketAddr;

use crate::models::project::ProjectStatus;
use crate::state::AppState;
use crate::views::{self, ContentType, ViewsService};
use crate::VERSION;

#[derive(Template)]
#[template(path = "projects/list.html")]
struct ProjectListTemplate<'a> {
    title: &'a str,
    nav_path: &'a str,
    version: &'a str,
    canonical_url: String,
    og_type: &'a str,
    og_image: Option<&'a str>,
    projects: Vec<ProjectItem<'a>>,
}

#[derive(Template)]
#[template(path = "projects/show.html")]
struct ProjectShowTemplate<'a> {
    title: &'a str,
    nav_path: &'a str,
    version: &'a str,
    canonical_url: String,
    og_type: &'a str,
    og_image: Option<&'a str>,
    description: Option<&'a str>,
    status: &'a str,
    github_url: Option<&'a str>,
    demo_url: Option<&'a str>,
    tags: &'a [String],
    content: &'a str,
    cover_image: Option<&'a str>,
    views_count: Option<String>,
}

struct ProjectItem<'a> {
    title: &'a str,
    slug: &'a str,
    description: Option<&'a str>,
    status: &'a str,
    github_url: Option<&'a str>,
    tags: &'a [String],
    cover_image: Option<&'a str>,
    views_count: Option<String>,
}

fn status_label(status: &ProjectStatus) -> &'static str {
    match status {
        ProjectStatus::Active => "Active",
        ProjectStatus::Completed => "Completed",
        ProjectStatus::Archived => "Archived",
        ProjectStatus::Planned => "Planned",
    }
}

pub async fn list(State(state): State<AppState>) -> Html<String> {
    let content = state.content.read().await;
    let all_projects = content.all_projects();

    // Batch fetch view counts if Redis is available
    let view_counts: Vec<Option<String>> = if let Some(ref redis) = state.redis {
        let service = ViewsService::new(redis.clone());
        let slugs: Vec<&str> = all_projects
            .iter()
            .map(|p| p.metadata.slug.as_str())
            .collect();
        match service.get_counts(ContentType::Project, &slugs).await {
            Ok(counts) => counts
                .into_iter()
                .map(|c| Some(views::format_count(c)))
                .collect(),
            Err(_) => vec![None; all_projects.len()],
        }
    } else {
        vec![None; all_projects.len()]
    };

    let projects: Vec<_> = all_projects
        .into_iter()
        .zip(view_counts)
        .map(|(p, views_count)| ProjectItem {
            title: &p.metadata.title,
            slug: &p.metadata.slug,
            description: p.metadata.description.as_deref(),
            status: status_label(&p.metadata.status),
            github_url: p.metadata.github_url.as_deref(),
            tags: &p.metadata.tags,
            cover_image: p.metadata.cover_image.as_deref(),
            views_count,
        })
        .collect();

    let template = ProjectListTemplate {
        title: "Projects",
        nav_path: "/projects",
        version: VERSION,
        canonical_url: format!("{}/projects", state.config.site_url),
        og_type: "website",
        og_image: None,
        projects,
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

    let project = content.projects.get(&slug).ok_or(StatusCode::NOT_FOUND)?;

    // Get views count and record view if Redis is available
    let views_count = if let Some(ref redis) = state.redis {
        let service = ViewsService::new(redis.clone());

        // Get current count first (for display)
        let count = service
            .get_count(ContentType::Project, &slug)
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
                    .record_view(ContentType::Project, &slug_owned, &ip, ua.as_deref())
                    .await;
            });
        }

        Some(views::format_count(count))
    } else {
        None
    };

    let cover_image = project.metadata.cover_image.as_deref();

    let template = ProjectShowTemplate {
        title: &project.metadata.title,
        nav_path: "/projects",
        version: VERSION,
        canonical_url: format!("{}/projects/{}", state.config.site_url, slug),
        og_type: "website",
        og_image: cover_image,
        description: project.metadata.description.as_deref(),
        status: status_label(&project.metadata.status),
        github_url: project.metadata.github_url.as_deref(),
        demo_url: project.metadata.demo_url.as_deref(),
        tags: &project.metadata.tags,
        content: &project.content_html,
        cover_image,
        views_count,
    };

    Ok(Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    ))
}
