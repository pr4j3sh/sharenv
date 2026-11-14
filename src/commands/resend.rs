use std::error::Error;

use crate::{
    api::client::ApiClient,
    commands::resolve_email,
    settings::AppConfig,
    utils::output::sanitize,
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
