use std::{error::Error, fmt};

use serde_json::Value;

#[derive(Debug)]
pub struct ApiError {
    pub status: reqwest::StatusCode,
    pub message: String,
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
    if let Ok(value) = serde_json::from_str::<Value>(body) {
        if let Some(detail) = value.get("detail") {
            return match detail {
                Value::String(message) => message.clone(),
                other => other.to_string(),
            };
        }
    }

    if body.trim().is_empty() {
        status.to_string()
    } else {
        body.to_string()
    }
}

/// Consume a non-success response and return the status and readable message.
pub async fn api_error(response: reqwest::Response) -> ApiError {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    ApiError {
        status,
        message: api_error_message(status, &body),
    }
}
