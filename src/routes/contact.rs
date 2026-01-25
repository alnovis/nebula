use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    Form,
};
use serde::Deserialize;

use crate::state::AppState;
use crate::turnstile;

#[derive(Template)]
#[template(path = "contact.html")]
struct ContactTemplate<'a> {
    title: &'a str,
    nav_path: &'a str,
    author_email: &'a str,
    error: Option<&'a str>,
    form_name: &'a str,
    form_email: &'a str,
    form_subject: &'a str,
    form_message: &'a str,
    turnstile_site_key: Option<&'a str>,
}

#[derive(Template)]
#[template(path = "contact_success.html")]
struct ContactSuccessTemplate<'a> {
    title: &'a str,
    nav_path: &'a str,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ContactFormData {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub message: String,
    // Honeypot field for spam protection
    #[serde(default)]
    pub website: String,
    // Cloudflare Turnstile response
    #[serde(default, rename = "cf-turnstile-response")]
    pub turnstile_response: String,
}

pub async fn show(State(state): State<AppState>) -> Html<String> {
    let template = ContactTemplate {
        title: "Contact",
        nav_path: "/contact",
        author_email: &state.config.author_email,
        error: None,
        form_name: "",
        form_email: "",
        form_subject: "",
        form_message: "",
        turnstile_site_key: state.config.turnstile_site_key.as_deref(),
    };

    Html(template.render().unwrap_or_else(|e| format!("Error: {}", e)))
}

pub async fn submit(
    State(state): State<AppState>,
    Form(form): Form<ContactFormData>,
) -> impl IntoResponse {
    // Check honeypot - if filled, it's likely a bot
    if !form.website.is_empty() {
        tracing::warn!("Honeypot triggered, rejecting submission");
        // Return success to not reveal the honeypot
        let template = ContactSuccessTemplate {
            title: "Message Sent",
            nav_path: "/contact",
        };
        return Html(template.render().unwrap_or_else(|e| format!("Error: {}", e)));
    }

    // Verify Turnstile CAPTCHA if configured
    if state.config.turnstile_configured() {
        let secret = state.config.turnstile_secret_key.as_ref().unwrap();

        if form.turnstile_response.is_empty() {
            return render_error(&state, "Please complete the CAPTCHA verification", &form);
        }

        match turnstile::verify(secret, &form.turnstile_response, None).await {
            Ok(response) => {
                if !response.success {
                    tracing::warn!("Turnstile verification failed: {:?}", response.error_codes);
                    return render_error(&state, "CAPTCHA verification failed. Please try again.", &form);
                }
            }
            Err(e) => {
                tracing::error!("Turnstile API error: {}", e);
                return render_error(&state, "CAPTCHA verification error. Please try again.", &form);
            }
        }
    }

    // Validate required fields
    if form.name.trim().is_empty() {
        return render_error(&state, "Name is required", &form);
    }

    if form.email.trim().is_empty() || !form.email.contains('@') {
        return render_error(&state, "Valid email is required", &form);
    }

    if form.message.trim().is_empty() {
        return render_error(&state, "Message is required", &form);
    }

    // Send the email
    let subject = if form.subject.trim().is_empty() {
        None
    } else {
        Some(form.subject.as_str())
    };

    match state
        .email
        .send_contact_message(&form.name, &form.email, subject, &form.message)
        .await
    {
        Ok(_) => {
            let template = ContactSuccessTemplate {
                title: "Message Sent",
                nav_path: "/contact",
            };
            Html(template.render().unwrap_or_else(|e| format!("Error: {}", e)))
        }
        Err(e) => {
            tracing::error!("Failed to send contact email: {}", e);
            render_error(&state, "Failed to send message. Please try again later.", &form)
        }
    }
}

fn render_error(state: &AppState, error: &str, form: &ContactFormData) -> Html<String> {
    let template = ContactTemplate {
        title: "Contact",
        nav_path: "/contact",
        author_email: &state.config.author_email,
        error: Some(error),
        form_name: &form.name,
        form_email: &form.email,
        form_subject: &form.subject,
        form_message: &form.message,
        turnstile_site_key: state.config.turnstile_site_key.as_deref(),
    };
    Html(template.render().unwrap_or_else(|e| format!("Error: {}", e)))
}
