use askama::Template;
use axum::extract::State;
use axum::response::Html;

use crate::state::AppState;
use crate::VERSION;

#[derive(Template)]
#[template(path = "about.html")]
struct AboutTemplate<'a> {
    title: &'a str,
    nav_path: &'a str,
    version: &'a str,
    canonical_url: String,
    og_type: &'a str,
    og_image: Option<&'a str>,
    author_name: &'a str,
}

pub async fn show(State(state): State<AppState>) -> Html<String> {
    let template = AboutTemplate {
        title: "About",
        nav_path: "/about",
        version: VERSION,
        canonical_url: format!("{}/about", state.config.site_url),
        og_type: "website",
        og_image: None,
        author_name: &state.config.author_name,
    };

    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
}
