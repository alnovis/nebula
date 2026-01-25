use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::Config;

const RESEND_API_URL: &str = "https://api.resend.com/emails";

/// Email service for sending contact form messages via Resend API
#[derive(Clone)]
pub struct EmailService {
    client: reqwest::Client,
    resend_api_key: Option<String>,
    from_email: String,
    contact_email: String,
    site_title: String,
}

#[derive(Serialize)]
struct ResendEmail<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    reply_to: &'a str,
    text: &'a str,
}

#[derive(Deserialize)]
struct ResendError {
    message: String,
}

impl EmailService {
    /// Create a new email service from config
    pub fn new(config: &Config) -> Self {
        let from_email = format!("Contact Form <notifications@{}>",
            config.site_url
                .trim_start_matches("https://")
                .trim_start_matches("http://")
        );

        if config.resend_configured() {
            tracing::info!("Email service configured with Resend API");
        } else {
            tracing::warn!("RESEND_API_KEY not configured, contact form emails will be logged only");
        }

        Self {
            client: reqwest::Client::new(),
            resend_api_key: config.resend_api_key.clone(),
            from_email,
            contact_email: config.contact_email.clone(),
            site_title: config.site_title.clone(),
        }
    }

    /// Send a contact form message
    pub async fn send_contact_message(
        &self,
        name: &str,
        email: &str,
        subject: Option<&str>,
        message: &str,
    ) -> Result<()> {
        let subject_line = subject
            .map(|s| format!("[{}] Contact: {}", self.site_title, s))
            .unwrap_or_else(|| format!("[{}] New contact message", self.site_title));

        let body = format!(
            "New contact form submission:\n\n\
             Name: {}\n\
             Email: {}\n\
             Subject: {}\n\n\
             Message:\n{}\n",
            name,
            email,
            subject.unwrap_or("(not specified)"),
            message
        );

        if let Some(api_key) = &self.resend_api_key {
            let email_payload = ResendEmail {
                from: &self.from_email,
                to: &self.contact_email,
                subject: &subject_line,
                reply_to: email,
                text: &body,
            };

            let response = self.client
                .post(RESEND_API_URL)
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&email_payload)
                .send()
                .await
                .context("Failed to send request to Resend API")?;

            if response.status().is_success() {
                tracing::info!("Contact email sent via Resend from {} <{}>", name, email);
            } else {
                let error: ResendError = response.json().await
                    .unwrap_or(ResendError { message: "Unknown error".to_string() });
                anyhow::bail!("Resend API error: {}", error.message);
            }
        } else {
            // Log the message when Resend is not configured
            tracing::info!(
                "Contact form submission (email not configured):\n{}",
                body
            );
        }

        Ok(())
    }

    /// Check if email sending is available
    pub fn is_available(&self) -> bool {
        self.resend_api_key.is_some()
    }
}
