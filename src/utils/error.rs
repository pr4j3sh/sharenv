use std::error::Error;

use serde_json::Value;

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

/// Consume a non-success response and return it as a boxed error.
pub async fn api_error(response: reqwest::Response) -> Box<dyn Error> {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    api_error_message(status, &body).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_string_detail() {
        assert_eq!(
            api_error_message(
                reqwest::StatusCode::CONFLICT,
                r#"{"detail":"project already exists"}"#,
            ),
            "project already exists"
        );
    }

    #[test]
    fn stringifies_non_string_detail() {
        assert_eq!(
            api_error_message(
                reqwest::StatusCode::UNPROCESSABLE_ENTITY,
                r#"{"detail":[{"msg":"invalid name"}]}"#,
            ),
            r#"[{"msg":"invalid name"}]"#
        );
    }

    #[test]
    fn falls_back_to_body() {
        assert_eq!(
            api_error_message(reqwest::StatusCode::BAD_GATEWAY, "bad gateway"),
            "bad gateway"
        );
    }

    #[test]
    fn falls_back_to_status_when_empty() {
        assert_eq!(
            api_error_message(reqwest::StatusCode::CONFLICT, ""),
            "409 Conflict"
        );
    }
}
