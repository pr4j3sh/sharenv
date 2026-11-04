use std::error::Error;

use crate::{
    api::client::ApiClient, settings::AppConfig, utils::prompt::prompt_credentials, utils::token,
};

pub async fn run(client: &ApiClient, config: &AppConfig) -> Result<(), Box<dyn Error>> {
    let (email, password) = prompt_credentials()?;

    let auth = client.register(email.clone(), password).await?;

    if let Err(error) = token::save_token(&config.auth, &auth.token) {
        return Err(
            format!("registered successfully, but failed to save the token: {error}").into(),
        );
    }

    println!("Registered and saved token for '{email}'.");

    Ok(())
}
