use std::error::Error;

use crate::{
    settings::AppConfig,
    utils::{jwt, token},
};

pub async fn run(config: &AppConfig) -> Result<(), Box<dyn Error>> {
    let base_url = &config.api_config.base_url;
    let profile = config.profile.as_deref();

    let token = token::get_token(&config.auth, base_url, profile)?.ok_or_else(|| {
        format!(
            "not logged in; run '{} login' first",
            env!("CARGO_PKG_NAME")
        )
    })?;

    // Local, unverified read of the access token — the server still enforces
    // everything; this is a UX view only.
    let subject = jwt::subject(&token).unwrap_or_else(|| "<unreadable>".to_string());
    let expiry_info = match jwt::expiry(&token) {
        Some(exp) => {
            let remaining = exp - chrono::Utc::now().timestamp();

            if remaining <= 0 {
                "expired (run 'sharenv login' or any command to refresh it)".to_string()
            } else {
                format!("expires in {}m", remaining / 60)
            }
        }
        None => "(no expiry claim)".to_string(),
    };

    let slot = token::account_for(&config.auth, base_url, profile);
    let pending = token::get_pending_verify(&config.auth, base_url, profile)?.is_some();

    if config.json_output {
        let subject = jwt::subject(&token).unwrap_or_default();
        let expiry = jwt::expiry(&token);

        crate::utils::output::json(&serde_json::json!({
            "user": subject,
            "expires_at": expiry.map(|exp| chrono::DateTime::<chrono::Utc>::from_timestamp(exp, 0))
                .map(|dt| dt.map(|d| d.to_rfc3339()))
                .flatten(),
            "base_url": base_url,
            "profile": profile.unwrap_or("(default)"),
            "keyring_slot": slot,
            "pending_verification": pending,
        }));

        return Ok(());
    }

    println!("user:     {subject}");
    println!("session:  {expiry_info}");
    println!("api:      {base_url}");
    println!("profile:  {}", profile.unwrap_or("(default)"));
    println!("keyring:  {slot}");
    println!(
        "pending verification: {}",
        if pending { "yes (run 'sharenv verify')" } else { "no" }
    );

    Ok(())
}
