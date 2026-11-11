use std::error::Error;

use zeroize::Zeroize;

use crate::{
    api::client::ApiClient,
    settings::AppConfig,
    utils::output::sanitize,
    utils::prompt::prompt_line,
    utils::token,
    utils::validation::validate_email,
};

pub async fn run(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    let email = resolve_email(config, args)?;

    println!("Verifying account '{}'", sanitize(&email));

    let mut otp = prompt_line("code: ")?;

    if otp.is_empty() {
        otp.zeroize();
        return Err("code cannot be empty".into());
    }

    let result = verify_flow(config, &email, &otp).await;
    otp.zeroize();

    result
}

/// Verify an account with an OTP and save the resulting session.
pub async fn verify_flow(
    config: &AppConfig,
    email: &str,
    otp: &str,
) -> Result<(), Box<dyn Error>> {
    let base_url = &config.api_config.base_url;
    let profile = config.profile.as_deref();
    let client = ApiClient::new(&config.api_config)?;
    let mut tokens = client.verify(email, otp).await?;

    token::save_session(&config.auth, base_url, profile, &tokens.token, &tokens.refresh_token)
        .map_err(|error| format!("verification succeeded, but failed to save tokens: {error}"))?;
    let _ = token::clear_pending_verify(&config.auth, base_url, profile);

    tokens.token.zeroize();
    tokens.refresh_token.zeroize();

    println!("Verified and logged in.");

    Ok(())
}

fn resolve_email(config: &AppConfig, args: &[String]) -> Result<String, Box<dyn Error>> {
    if let Some(email) = args.first() {
        let email = email.trim().to_lowercase();
        validate_email(&email)?;
        return Ok(email);
    }

    let base_url = &config.api_config.base_url;
    let profile = config.profile.as_deref();

    if let Some(email) = token::get_pending_verify(&config.auth, base_url, profile)? {
        return Ok(email);
    }

    let input = prompt_line("email: ")?;
    let email = input.trim().to_lowercase();
    validate_email(&email)?;

    Ok(email)
}
