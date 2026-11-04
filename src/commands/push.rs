use std::error::Error;

use crate::{
    api::client::ApiClient,
    settings::AppConfig,
    utils::{
        args::parse_value_arg, env_file::env_file_path, env_parser::read_env_file,
        error::is_conflict_error, local_config, token, validation::validate_environment_name,
    },
};

pub async fn run(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    let env_name = parse_value_arg(args, "--environment", "-e", "environment name")?
        .ok_or("missing --environment/-e <env_name>")?;

    validate_environment_name(&env_name)?;

    let local = local_config::load(config)
        .map_err(|_| format!("no {} found", config.environment_files.config_file_path))?;

    if local.project.id.is_empty() {
        return Err(format!(
            "project id is missing from {}",
            config.environment_files.config_file_path
        )
        .into());
    }

    let token = token::get_token(&config.auth)?.ok_or_else(|| {
        format!(
            "not logged in; run '{} login' first",
            env!("CARGO_PKG_NAME")
        )
    })?;
    let client = ApiClient::new(&config.api_config).with_token(token);

    let environments = client
        .get_environments(&local.project.id, &env_name)
        .await?;
    let environment = environments
        .into_iter()
        .find(|environment| environment.name == env_name)
        .ok_or_else(|| format!("environment '{env_name}' not found"))?;

    let path = env_file_path(config, &env_name);
    let variables =
        read_env_file(&path).map_err(|error| format!("failed to read {path}: {error}"))?;

    if variables.is_empty() {
        println!("No variables found in {path}");
        return Ok(());
    }

    let mut pushed = 0;
    let mut skipped = 0;

    for variable in variables {
        match client
            .create_variable(&environment._id, variable.name.clone(), variable.value)
            .await
        {
            Ok(_) => {
                pushed += 1;
                println!("+ {}", variable.name);
            }
            Err(error) if is_conflict_error(error.as_ref()) => {
                skipped += 1;
                println!("= {} already exists, skipped", variable.name);
            }
            Err(error) => {
                return Err(format!("failed to push '{}': {error}", variable.name).into());
            }
        }
    }

    println!("Pushed {pushed} variable(s) to '{env_name}' ({skipped} skipped)");

    Ok(())
}
