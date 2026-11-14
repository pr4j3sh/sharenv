use std::{error::Error, path::Path};

use crate::{
    settings::AppConfig,
    utils::{
        args::take_positional,
        env_file::env_file_path,
        error::is_conflict_error,
        fs_secure, local_config,
        session,
        validation::validate_environment_name,
    },
};

pub async fn run(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    let environment_name = take_positional(args)?
        .ok_or("missing environment name; usage: add <environment>")?;

    validate_environment_name(&environment_name)?;

    let project_id = local_config::project_id(config)?;

    let client = session::authenticated_client(config).await?;

    let environment = match client
        .create_environment(&project_id, environment_name.clone())
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
        fs_secure::atomic_write_secure(Path::new(&path), "").map_err(|error| {
            format!("environment created remotely, but failed to create {path}: {error}")
        })?;
        println!("Created {path}");
    }

    Ok(())
}
