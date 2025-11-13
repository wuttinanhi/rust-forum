use crate::errors::turnstile::TurnstileError;
use serde::Deserialize;
use std::collections::HashMap;

/// The Cloudflare Turnstile verification endpoint.
const VERIFY_ENDPOINT: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";

/// Represents the JSON response from the Cloudflare siteverify API.
/// We use `#[serde(default)]` for fields that may not always be present,
/// especially `error_codes`, which is empty on success.
#[derive(Deserialize, Debug)]
pub struct TurnstileResponse {
    success: bool,

    #[serde(rename = "error-codes")]
    #[serde(default)]
    #[allow(dead_code)]
    error_codes: Vec<String>,

    #[serde(default)]
    #[allow(dead_code)]
    challenge_ts: Option<String>,

    #[serde(default)]
    #[allow(dead_code)]
    hostname: Option<String>,

    #[serde(default)]
    #[allow(dead_code)]
    action: Option<String>,

    #[serde(default)]
    #[allow(dead_code)]
    cdata: Option<String>,
}

/// Validates a Cloudflare Turnstile token using reqwest.
///
/// # Arguments
/// * `client` - A reference to a `reqwest::Client`. It's crucial to reuse the client.
/// * `secret_key` - Your Turnstile secret key.
/// * `token` - The `cf-turnstile-response` token from the client.
/// * `remote_ip` - An optional IP address of the end user.
///
/// # Returns
/// A `Result` containing:
/// * `Ok(TurnstileResponse)` - If the API call was successful. You must still check
///                            `response.success` to see if validation passed.
/// * `Err(reqwest::Error)` - If the request to the Cloudflare API failed.
async fn validate_turnstile(
    secret_key: &str,
    token: &str,
    remote_ip: Option<&str>,
) -> Result<TurnstileResponse, reqwest::Error> {
    let http_client = reqwest::Client::new();

    // 1. Build the form parameters
    // We use a HashMap to easily handle the optional `remoteip`.
    let mut params = HashMap::new();
    params.insert("secret", secret_key);
    params.insert("response", token);
    if let Some(ip) = remote_ip {
        params.insert("remoteip", ip);
    }

    // 2. Send the POST request
    // The `.form()` method automatically sets the content-type
    // to `application/x-www-form-urlencoded`.
    let response = http_client
        .post(VERIFY_ENDPOINT)
        .form(&params)
        .send()
        .await?;

    // 3. Deserialize the JSON response
    // `.json()` will parse the response and return an Err
    // if deserialization fails or if the request had a non-200 status.
    let validation_result = response.json::<TurnstileResponse>().await?;

    Ok(validation_result)
}

pub async fn validate_turnstile_wrapper(response_token: &str) -> Result<bool, TurnstileError> {
    let turnstile_secret_key = std::env::var("CLOUDFLARE_TURNSTILE_SITE_KEY");

    if let Ok(turnstile_secret_key) = turnstile_secret_key {
        if !response_token.is_empty() {
            let result = validate_turnstile(&turnstile_secret_key, response_token, None)
                .await
                .map_err(|e| TurnstileError {
                    message: format!("Failed to validate Turnstile token: {}", e),
                })?;

            Ok(result.success)
        } else {
            Err(TurnstileError {
                message: "Turnstile response token is missing.".to_string(),
            })
        }
    } else {
        Ok(true)
    }
}
