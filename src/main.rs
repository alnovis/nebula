use std::net::SocketAddr;

use anyhow::Result;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use nebula::{config::Config, create_app};

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("Nebula starting...");

    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nebula=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    eprintln!("Loading config...");

    // Load configuration
    dotenvy::dotenv().ok();
    let config = match Config::from_env() {
        Ok(c) => {
            eprintln!("Config loaded: port={}", c.port);
            c
        }
        Err(e) => {
            eprintln!("Config error: {}", e);
            return Err(e);
        }
    };

    eprintln!("Creating app...");

    // Create application
    let app = match create_app(&config).await {
        Ok(a) => {
            eprintln!("App created");
            a
        }
        Err(e) => {
            eprintln!("App creation error: {}", e);
            return Err(e);
        }
    };

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    eprintln!("Binding to {}...", addr);
    let listener = TcpListener::bind(addr).await?;

    eprintln!("Nebula started at http://{}", addr);
    info!("Nebula started at http://{}", addr);
    info!("Site: {}", config.site_url);

    axum::serve(listener, app).await?;

    Ok(())
}
