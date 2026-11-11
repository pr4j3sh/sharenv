use std::{error::Error, fmt};

use serde_json::Value;

#[derive(Debug)]
pub struct ApiError {
    pub status: reqwest::StatusCode,
    pub message: String,
    #[allow(dead_code)]
    pub retry_after: Option<String>,
}

impl ApiError {
    pub fn is_conflict(&self) -> bool {
        self.status == reqwest::StatusCode::CONFLICT
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ApiError {}

pub fn is_conflict_error(error: &(dyn Error + 'static)) -> bool {
    error
        .downcast_ref::<ApiError>()
        .is_some_and(ApiError::is_conflict)
}

/// Pull a readable message out of an API error body.
pub fn api_error_message(status: reqwest::StatusCode, body: &str) -> String {
    let message = if let Ok(value) = serde_json::from_str::<Value>(body) {
        match value.get("detail") {
            Some(Value::String(message)) => message.clone(),
            Some(other) => other.to_string(),
            None if body.trim().is_empty() => status.to_string(),
            None => body.to_string(),
        }
    } else if body.trim().is_empty() {
        status.to_string()
    } else {
        body.to_string()
    };

    crate::utils::output::sanitize(&message)
}

/// Consume a non-success response and return the status and readable message.
pub async fn api_error(response: reqwest::Response) -> ApiError {
    let status = response.status();
    let retry_after = response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let body = response.text().await.unwrap_or_default();

    let mut message = api_error_message(status, &body);

    match status {
        reqwest::StatusCode::UNAUTHORIZED => {
            if message == "invalid credentials" {
                // A failed login attempt, not an expired session.
                message = "invalid email or password".to_string();
            } else if message == "failed to verify otp" {
                // The server cannot (and will not) say whether this was a bad
                // code, an unknown account, or an already-verified account.
                message = "invalid or expired code (or this account is already verified)".to_string();
            } else {
                message.push_str(" (your session expired or was revoked; run 'sharenv login')");
            }
        }
        reqwest::StatusCode::FORBIDDEN => {
            if message == "email_not_verified" {
                message.push_str(" (your email is not verified; run 'sharenv verify')");
            } else if message.contains("re-authentication") {
                message.push_str(" (your password is incorrect)");
            } else {
                message.push_str(" (you do not have access; your role or membership is insufficient)");
            }
        }
        reqwest::StatusCode::NOT_FOUND => {
            message.push_str(" (not found, or you are not a member)");
        }
        reqwest::StatusCode::TOO_MANY_REQUESTS => match &retry_after {
            Some(seconds) => message.push_str(&format!(" (retry after {seconds} seconds)")),
            None => message.push_str(" (retry later)"),
        },
        _ => {}
    }

    ApiError {
        status,
        message,
        retry_after,
    }
}
