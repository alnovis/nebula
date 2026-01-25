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
    // SMTP configuration for contact form
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_user: Option<String>,
    pub smtp_password: Option<String>,
    pub contact_email: String,
    // Resend API for email sending
    pub resend_api_key: Option<String>,
    // Cloudflare Turnstile CAPTCHA
    pub turnstile_site_key: Option<String>,
    pub turnstile_secret_key: Option<String>,
    // Admin secret for content reload
    pub admin_secret: Option<String>,
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
            database_url: env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            content_dir: env::var("CONTENT_DIR").unwrap_or_else(|_| "./content".into()),
            site_url: env::var("SITE_URL").unwrap_or_else(|_| "http://localhost:3000".into()),
            site_title: env::var("SITE_TITLE").unwrap_or_else(|_| "Nebula".into()),
            site_description: env::var("SITE_DESCRIPTION")
                .unwrap_or_else(|_| "Personal blog and project showcase".into()),
            author_name: env::var("AUTHOR_NAME").unwrap_or_else(|_| "Author".into()),
            author_email: env::var("AUTHOR_EMAIL").unwrap_or_else(|_| "author@example.com".into()),
            smtp_host: env::var("SMTP_HOST").ok(),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".into())
                .parse()
                .unwrap_or(587),
            smtp_user: env::var("SMTP_USER").ok(),
            smtp_password: env::var("SMTP_PASSWORD").ok(),
            contact_email: env::var("CONTACT_EMAIL").unwrap_or_else(|_| {
                env::var("AUTHOR_EMAIL").unwrap_or_else(|_| "author@example.com".into())
            }),
            resend_api_key: env::var("RESEND_API_KEY").ok(),
            turnstile_site_key: env::var("TURNSTILE_SITE_KEY").ok(),
            turnstile_secret_key: env::var("TURNSTILE_SECRET_KEY").ok(),
            admin_secret: env::var("ADMIN_SECRET").ok(),
        })
    }

    /// Check if SMTP is configured
    pub fn smtp_configured(&self) -> bool {
        self.smtp_host.is_some() && self.smtp_user.is_some() && self.smtp_password.is_some()
    }

    /// Check if Resend API is configured
    pub fn resend_configured(&self) -> bool {
        self.resend_api_key.is_some()
    }

    /// Check if Turnstile CAPTCHA is configured
    pub fn turnstile_configured(&self) -> bool {
        self.turnstile_site_key.is_some() && self.turnstile_secret_key.is_some()
    }
}
