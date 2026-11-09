use std::error::Error;

use crate::{
    api::client::ApiClient, settings::AppConfig, utils::output::sanitize,
    utils::prompt::prompt_credentials, utils::token,
};

pub async fn run(client: &ApiClient, config: &AppConfig) -> Result<(), Box<dyn Error>> {
    let (email, password) = prompt_credentials()?;

    let auth = client.login(email.clone(), password).await?;

    if let Err(error) = token::save_token(&config.auth, &auth.token) {
        return Err(format!("login successfully, but failed to save the token: {error}").into());
    }

    println!("logged in and saved token for '{}'.", sanitize(&email));

    Ok(())
}
