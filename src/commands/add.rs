use std::{error::Error, fs::File, path::Path};

use crate::{
    settings::AppConfig,
    utils::{
        args::take_positional,
        env_file::env_file_path,
        error::is_conflict_error,
        local_config,
        session,
        validation::validate_environment_name,
    },
};

pub async fn run(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    let environment_name = take_positional(args)?
        .ok_or("missing environment name; usage: add <environment>")?;

    validate_environment_name(&environment_name)?;

    let local =
        local_config::load(config).map_err(|_| format!("no {} found", config.config_file_path))?;

    if local.project.id.is_empty() {
        return Err(format!(
            "project id is missing from {}; run '{} init' first",
            config.config_file_path,
            env!("CARGO_PKG_NAME")
        )
        .into());
    }

    let client = session::authenticated_client(config).await?;

    let environment = match client
        .create_environment(&local.project.id, environment_name.clone())
        .await
    {
        Ok(environment) => environment,
        Err(error) if is_conflict_error(error.as_ref()) => {
            println!("Environment '{environment_name}' already exists.");
            return Ok(());
        }
        Err(error) => return Err(error),
    };

    println!(
        "Created environment '{environment_name}' ({})",
        environment.environment_id
    );

    let path = env_file_path(config, &environment_name);
    if !Path::new(&path).exists() {
        File::create(&path).map_err(|error| {
            format!("environment created remotely, but failed to create {path}: {error}")
        })?;
        println!("Created {path}");
    }

    Ok(())
}
