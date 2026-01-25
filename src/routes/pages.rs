use askama::Template;
use axum::extract::State;
use axum::response::Html;

use crate::state::AppState;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    title: &'a str,
    nav_path: &'a str,
    recent_posts: Vec<PostSummary<'a>>,
    featured_projects: Vec<ProjectSummary<'a>>,
}

#[derive(Template)]
#[template(path = "about.html")]
struct AboutTemplate<'a> {
    title: &'a str,
    nav_path: &'a str,
    author_name: &'a str,
}

struct PostSummary<'a> {
    title: &'a str,
    slug: &'a str,
    description: Option<&'a str>,
    date: String,
}

struct ProjectSummary<'a> {
    title: &'a str,
    slug: &'a str,
    description: Option<&'a str>,
    status: &'a str,
}

pub async fn index(State(state): State<AppState>) -> Html<String> {
    let content = state.content.read().await;

    let recent_posts: Vec<_> = content
        .published_posts()
        .into_iter()
        .take(5)
        .map(|p| PostSummary {
            title: &p.metadata.title,
            slug: &p.metadata.slug,
            description: p.metadata.description.as_deref(),
            date: p.metadata.date.format("%Y-%m-%d").to_string(),
        })
        .collect();

    let featured_projects: Vec<_> = content
        .featured_projects()
        .into_iter()
        .take(3)
        .map(|p| ProjectSummary {
            title: &p.metadata.title,
            slug: &p.metadata.slug,
            description: p.metadata.description.as_deref(),
            status: match p.metadata.status {
                crate::models::project::ProjectStatus::Active => "Active",
                crate::models::project::ProjectStatus::Completed => "Completed",
                crate::models::project::ProjectStatus::Archived => "Archived",
                crate::models::project::ProjectStatus::Planned => "Planned",
            },
        })
        .collect();

    let template = IndexTemplate {
        title: &state.config.site_title,
        nav_path: "/",
        recent_posts,
        featured_projects,
    };

    Html(template.render().unwrap_or_else(|e| format!("Error: {}", e)))
}

pub async fn about(State(state): State<AppState>) -> Html<String> {
    let template = AboutTemplate {
        title: "About",
        nav_path: "/about",
        author_name: &state.config.author_name,
    };

    Html(template.render().unwrap_or_else(|e| format!("Error: {}", e)))
}
