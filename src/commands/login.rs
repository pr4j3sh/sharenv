use std::{
    error::Error,
    io::{self, Write},
};

use crate::{api::client::ApiClient, settings::AppConfig, utils::token};

pub async fn run(client: &ApiClient, config: &AppConfig) -> Result<(), Box<dyn Error>> {
    print!("email: ");
    io::stdout().flush()?;

    let mut email = String::new();
    io::stdin().read_line(&mut email)?;
    let email = email.trim();

    if email.is_empty() {
        return Err("Email cannot be empty".into());
    }

    let password = rpassword::prompt_password("password: ")?;

    if password.is_empty() {
        return Err("Password cannot be empty".into());
    }

    let auth = client.login(email.to_string(), password).await?;

    if let Err(error) = token::save_token(&config.auth, &auth.token) {
        return Err(format!("login successfully, but failed to save the token: {error}").into());
    }

    println!("logged in and saved token for '{email}'.");

    Ok(())
}
