use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Html;

use crate::models::project::ProjectStatus;
use crate::state::AppState;

#[derive(Template)]
#[template(path = "projects/list.html")]
struct ProjectListTemplate<'a> {
    title: &'a str,
    nav_path: &'a str,
    projects: Vec<ProjectItem<'a>>,
}

#[derive(Template)]
#[template(path = "projects/show.html")]
struct ProjectShowTemplate<'a> {
    title: &'a str,
    nav_path: &'a str,
    description: Option<&'a str>,
    status: &'a str,
    github_url: Option<&'a str>,
    demo_url: Option<&'a str>,
    tags: &'a [String],
    content: &'a str,
}

struct ProjectItem<'a> {
    title: &'a str,
    slug: &'a str,
    description: Option<&'a str>,
    status: &'a str,
    github_url: Option<&'a str>,
    tags: &'a [String],
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

    let projects: Vec<_> = content
        .all_projects()
        .into_iter()
        .map(|p| ProjectItem {
            title: &p.metadata.title,
            slug: &p.metadata.slug,
            description: p.metadata.description.as_deref(),
            status: status_label(&p.metadata.status),
            github_url: p.metadata.github_url.as_deref(),
            tags: &p.metadata.tags,
        })
        .collect();

    let template = ProjectListTemplate {
        title: "Projects",
        nav_path: "/projects",
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
) -> Result<Html<String>, StatusCode> {
    let content = state.content.read().await;

    let project = content.projects.get(&slug).ok_or(StatusCode::NOT_FOUND)?;

    let template = ProjectShowTemplate {
        title: &project.metadata.title,
        nav_path: "/projects",
        description: project.metadata.description.as_deref(),
        status: status_label(&project.metadata.status),
        github_url: project.metadata.github_url.as_deref(),
        demo_url: project.metadata.demo_url.as_deref(),
        tags: &project.metadata.tags,
        content: &project.content_html,
    };

    Ok(Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    ))
}
