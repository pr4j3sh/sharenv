use std::error::Error;

use zeroize::Zeroize;

use crate::{
    api::client::ApiClient,
    settings::AppConfig,
    utils::output::sanitize,
    utils::prompt::prompt_credentials,
    utils::token,
};

pub async fn run(client: &ApiClient, config: &AppConfig) -> Result<(), Box<dyn Error>> {
    let (email, mut password) = prompt_credentials()?;

    let auth = match client.login(&email, &password).await {
        Ok(auth) => auth,
        Err(error) => {
            password.zeroize();

            if let Some(api_error) = error.downcast_ref::<crate::utils::error::ApiError>() {
                if api_error.status == reqwest::StatusCode::FORBIDDEN
                    && api_error.message.starts_with("email_not_verified")
                {
                    return Err(format!(
                        "'{}' is not verified yet. A verification code was emailed at registration; run '{} verify' to verify and log in.",
                        sanitize(&email),
                        env!("CARGO_PKG_NAME")
                    )
                    .into());
                }
            }

            return Err(error);
        }
    };

    password.zeroize();

    token::save_session(
        &config.auth,
        &config.api_config.base_url,
        config.profile.as_deref(),
        &auth.token,
        &auth.refresh_token,
    )?;

    let mut token = auth.token;
    let mut refresh_token = auth.refresh_token;
    token.zeroize();
    refresh_token.zeroize();

    println!("logged in and saved token for '{}'.", sanitize(&email));

    // With a mail provider configured the server blocks unverified logins
    // outright, so reaching this branch means no provider is set up and an
    // unverified login is expected.
    if !auth.email_verified {
        println!(
            "note: this server has no mail provider configured; email verification is disabled"
        );
    }

    Ok(())
}
