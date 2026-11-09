//! Local, unverified JWT claim reads.
//!
//! These values are used for display and UX decisions only. Authorization is
//! always enforced by the server.

use base64::{
    Engine as _,
    engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD},
};
use serde_json::Value;

pub fn subject(token: &str) -> Option<String> {
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .or_else(|_| URL_SAFE.decode(payload))
        .ok()?;
    let claims: Value = serde_json::from_slice(&bytes).ok()?;

    claims.get("sub")?.as_str().map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::subject;
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

    #[test]
    fn reads_subject_without_verifying() {
        let claims = serde_json::json!({ "sub": "user-123", "exp": 1 });
        let payload = URL_SAFE_NO_PAD.encode(claims.to_string());
        let token = format!("header.{payload}.signature");

        assert_eq!(subject(&token).as_deref(), Some("user-123"));
    }

    #[test]
    fn rejects_malformed_tokens() {
        assert_eq!(subject("not-a-token"), None);
        assert_eq!(subject("header.bm90LWpzb24.signature"), None);
    }
}
