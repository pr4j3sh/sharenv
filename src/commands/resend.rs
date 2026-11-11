use std::error::Error;

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
    let client = ApiClient::new(&config.api_config)?;

    client.resend(&email).await?;

    // The server deliberately returns the same response for unknown and
    // already-verified accounts, so say exactly what happened.
    println!(
        "If an unverified account exists for '{}', a new code was sent",
        sanitize(&email)
    );

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
