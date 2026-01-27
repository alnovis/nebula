/// Application version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod config;
pub mod content;
pub mod email;
pub mod models;
pub mod routes;
pub mod state;
pub mod turnstile;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::{config::Config, state::AppState};

/// Create the main application router
pub async fn create_app(config: &Config) -> Result<Router> {
    // Database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Load content from filesystem
    let content_store = content::ContentStore::load(&config.content_dir).await?;

    // Create email service
    let email_service = email::EmailService::new(config);

    // Create shared state
    let state = AppState::new(pool, content_store, config.clone(), email_service);

    // Build router
    let app = Router::new()
        // Pages
        .route("/", get(routes::pages::index))
        .route("/resume", get(routes::resume::show))
        .route("/projects", get(routes::projects::list))
        .route("/projects/:slug", get(routes::projects::show))
        // Blog
        .route("/blog", get(routes::blog::list))
        .route("/blog/:slug", get(routes::blog::show))
        // Contact
        .route("/contact", get(routes::contact::show))
        .route("/contact", post(routes::contact::submit))
        // Feeds
        .route("/rss.xml", get(routes::feeds::rss))
        .route("/sitemap.xml", get(routes::feeds::sitemap))
        .route("/robots.txt", get(routes::feeds::robots))
        // Health check
        .route("/health", get(routes::health::check))
        // Admin
        .route("/admin/reload", post(routes::admin::reload_content))
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        .route_service("/favicon.ico", ServeFile::new("static/favicon.ico"))
        // Middleware
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        // State
        .with_state(state);

    Ok(app)
}
