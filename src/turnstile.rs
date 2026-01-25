use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const TURNSTILE_VERIFY_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";

#[derive(Serialize)]
struct VerifyRequest<'a> {
    secret: &'a str,
    response: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    remoteip: Option<&'a str>,
}

#[derive(Deserialize, Debug)]
pub struct VerifyResponse {
    pub success: bool,
    #[serde(default)]
    pub error_codes: Vec<String>,
    #[serde(default)]
    pub challenge_ts: Option<String>,
    #[serde(default)]
    pub hostname: Option<String>,
}

/// Verify a Turnstile CAPTCHA response token
pub async fn verify(secret: &str, token: &str, remote_ip: Option<&str>) -> Result<VerifyResponse> {
    let client = reqwest::Client::new();

    let request = VerifyRequest {
        secret,
        response: token,
        remoteip: remote_ip,
    };

    let response = client
        .post(TURNSTILE_VERIFY_URL)
        .form(&request)
        .send()
        .await
        .context("Failed to send Turnstile verification request")?;

    let verify_response: VerifyResponse = response
        .json()
        .await
        .context("Failed to parse Turnstile verification response")?;

    Ok(verify_response)
}
