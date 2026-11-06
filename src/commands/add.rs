use std::error::Error;

use crate::{
    api::client::ApiClient,
    settings::AppConfig,
    utils::{args::parse_value_arg, local_config, token},
};

pub async fn run(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    let email =
        parse_value_arg(args, "--user", "-u", "email")?.ok_or("missing --user/-u <email>")?;

    let local =
        local_config::load(config).map_err(|_| format!("no {} found", config.config_file_path))?;

    if local.project.id.is_empty() {
        return Err(format!("project id is missing from {}", config.config_file_path).into());
    }

    let token = token::get_token(&config.auth)?.ok_or_else(|| {
        format!(
            "not logged in; run '{} login' first",
            env!("CARGO_PKG_NAME")
        )
    })?;
    let client = ApiClient::new(&config.api_config).with_token(token);

    let project = client.add_user(&local.project.id, email.clone()).await?;

    println!("Shared project '{}' with '{email}'.", project.project_id);
    println!(
        "They can pull it with: {} pull --pid {} -e <environment>",
        env!("CARGO_PKG_NAME"),
        project.project_id
    );

    Ok(())
}
