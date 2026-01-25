use askama::Template;
use axum::extract::State;
use axum::response::Html;

use crate::state::AppState;

#[derive(Template)]
#[template(path = "resume.html")]
struct ResumeTemplate<'a> {
    title: &'a str,
    nav_path: &'a str,
    author_name: &'a str,
    author_email: &'a str,
}

pub async fn show(State(state): State<AppState>) -> Html<String> {
    let template = ResumeTemplate {
        title: "Resume",
        nav_path: "/resume",
        author_name: &state.config.author_name,
        author_email: &state.config.author_email,
    };

    Html(template.render().unwrap_or_else(|e| format!("Error: {}", e)))
}
