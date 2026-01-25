use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::{config::Config, content::ContentStore, email::EmailService};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub content: Arc<RwLock<ContentStore>>,
    pub config: Config,
    pub email: EmailService,
}

impl AppState {
    pub fn new(pool: PgPool, content: ContentStore, config: Config, email: EmailService) -> Self {
        Self {
            pool,
            content: Arc::new(RwLock::new(content)),
            config,
            email,
        }
    }

    /// Reload content from filesystem
    pub async fn reload_content(&self) -> anyhow::Result<()> {
        let new_content = ContentStore::load(&self.config.content_dir).await?;
        let mut content = self.content.write().await;
        *content = new_content;
        Ok(())
    }
}
