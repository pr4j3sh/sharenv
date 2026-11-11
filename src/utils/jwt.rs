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
    let claims = claims(token)?;

    claims.get("sub")?.as_str().map(ToString::to_string)
}

/// The unverified `exp` claim as seconds since the epoch, if present.
pub fn expiry(token: &str) -> Option<i64> {
    let claims = claims(token)?;

    claims.get("exp")?.as_i64()
}

/// True when the token is missing an `exp` or expires within `within_seconds`.
pub fn expiring_within(token: &str, within_seconds: i64) -> bool {
    match expiry(token) {
        Some(exp) => exp <= chrono::Utc::now().timestamp() + within_seconds,
        None => true,
    }
}

fn claims(token: &str) -> Option<Value> {
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .or_else(|_| URL_SAFE.decode(payload))
        .ok()?;

    serde_json::from_slice(&bytes).ok()
}
