use std::error::Error;

use zeroize::Zeroize;

use crate::{
    api::client::ApiClient,
    commands::verify,
    settings::AppConfig,
    utils::output::sanitize,
    utils::prompt::{prompt_credentials, prompt_line},
    utils::token,
};

pub async fn run(client: &ApiClient, config: &AppConfig) -> Result<(), Box<dyn Error>> {
    let (email, mut password) = prompt_credentials()?;

    if password.chars().count() < 12 {
        password.zeroize();
        return Err("password must be at least 12 characters long".into());
    }

    let email = email.trim().to_lowercase();
    let auth = client.register(&email, &password).await?;
    password.zeroize();

    println!("Registered '{}'.", sanitize(&email));

    if !auth.verification_required {
        println!("Email verification is not enabled on this server; you can log in now.");
        return Ok(());
    }

    token::save_pending_verify(
        &config.auth,
        &config.api_config.base_url,
        config.profile.as_deref(),
        &email,
    )
    .map_err(|error| format!("failed to save your email for verification: {error}"))?;

    println!("A verification code was sent to '{}'.", sanitize(&email));

    let otp = prompt_line("Enter the code (or leave empty to verify later): ")?;

    if otp.is_empty() {
        println!(
            "Verify later with: {} verify",
            env!("CARGO_PKG_NAME")
        );
        return Ok(());
    }

    verify::verify_flow(config, &email, &otp).await
}
