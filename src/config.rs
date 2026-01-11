use anyhow::{Context, Result};
use std::env;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub content_dir: String,
    pub site_url: String,
    pub site_title: String,
    pub site_description: String,
    pub author_name: String,
    pub author_email: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .context("Invalid PORT")?,
            database_url: env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            content_dir: env::var("CONTENT_DIR")
                .unwrap_or_else(|_| "./content".into()),
            site_url: env::var("SITE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".into()),
            site_title: env::var("SITE_TITLE")
                .unwrap_or_else(|_| "Nebula".into()),
            site_description: env::var("SITE_DESCRIPTION")
                .unwrap_or_else(|_| "Personal blog and project showcase".into()),
            author_name: env::var("AUTHOR_NAME")
                .unwrap_or_else(|_| "Author".into()),
            author_email: env::var("AUTHOR_EMAIL")
                .unwrap_or_else(|_| "author@example.com".into()),
        })
    }
}
