/// Application version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod analytics;
pub mod config;
pub mod content;
pub mod email;
pub mod models;
pub mod routes;
pub mod state;
pub mod turnstile;
pub mod views;

use anyhow::Result;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use redis::Client as RedisClient;
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

    // Connect to Redis if configured
    let redis = if let Some(ref redis_url) = config.redis_url {
        match RedisClient::open(redis_url.as_str()) {
            Ok(client) => match client.get_connection_manager().await {
                Ok(manager) => {
                    tracing::info!("Connected to Redis for views counter");
                    Some(manager)
                }
                Err(e) => {
                    tracing::warn!("Failed to connect to Redis: {}. Views counter disabled.", e);
                    None
                }
            },
            Err(e) => {
                tracing::warn!("Invalid Redis URL: {}. Views counter disabled.", e);
                None
            }
        }
    } else {
        tracing::info!("Redis not configured. Views counter disabled.");
        None
    };

    // Create shared state
    let state = AppState::new(pool, content_store, config.clone(), email_service, redis);

    // Start analytics background tasks
    analytics::geoip::start_updater(
        state.pool.clone(),
        reqwest::Client::new(),
        config.geoip_url.clone(),
    );
    analytics::scheduler::start(state.clone());

    // Build router
    let app = Router::new()
        // Pages
        .route("/", get(routes::pages::index))
        .route("/about", get(routes::about::show))
        .route("/resume", get(routes::resume::show))
        .route("/resume/print", get(routes::resume::print))
        .route("/projects", get(routes::projects::list))
        .route("/projects/:slug", get(routes::projects::show))
        // Blog
        .route("/blog", get(routes::blog::list))
        .route("/blog/tag/:tag", get(routes::blog::by_tag))
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
        .route("/health/cdn", get(routes::health::cdn_check))
        .route("/health/cdn/report", post(routes::health::cdn_report))
        // Analytics
        .route(
            "/api/analytics/event",
            post(routes::analytics::record_event),
        )
        // Admin
        .route("/admin/reload", post(routes::admin::reload_content))
        .route("/admin/analytics", get(routes::admin::analytics_dashboard))
        .route(
            "/admin/analytics/report",
            get(routes::admin::analytics_report),
        )
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        .route_service("/favicon.ico", ServeFile::new("static/favicon.ico"))
        // Middleware
        .layer(middleware::from_fn_with_state(
            state.clone(),
            analytics::middleware::analytics_middleware,
        ))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        // State
        .with_state(state);

    Ok(app)
}
