use std::{error::Error, fs::File, path::Path};

use crate::{
    settings::AppConfig,
    utils::{
        args::{FlagSpec, parse_flags},
        env_file::env_file_path,
        error::is_conflict_error,
        local_config,
        session,
        validation::validate_environment_name,
    },
};

pub async fn run(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    let specs = [
        FlagSpec {
            key: "user",
            names: &["--user", "-u"],
            label: "email",
        },
        FlagSpec {
            key: "environment",
            names: &["--environment", "-e"],
            label: "environment name",
        },
    ];

    let mut flags = parse_flags(args, &specs)?;

    let email = flags.remove("user");
    let environment = flags.remove("environment");

    match (email, environment) {
        (Some(_), Some(_)) => Err("specify either --user/-u or --environment/-e, not both".into()),
        (None, None) => {
            Err("missing add target; use -e <environment_name> to create an environment".into())
        }
        (Some(_), None) => Err(format!(
            "sharing by email moved to the invitation flow; use '{} invite <email>' instead",
            env!("CARGO_PKG_NAME")
        )
        .into()),
        (None, Some(environment)) => add_environment(config, &environment).await,
    }
}

async fn add_environment(
    config: &AppConfig,
    environment_name: &str,
) -> Result<(), Box<dyn Error>> {
    validate_environment_name(environment_name)?;

    let local = load_local(config)?;
    let client = session::authenticated_client(config).await?;

    let environment = match client
        .create_environment(&local.project.id, environment_name.to_string())
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

    let path = env_file_path(config, environment_name);
    if !Path::new(&path).exists() {
        File::create(&path).map_err(|error| {
            format!("environment created remotely, but failed to create {path}: {error}")
        })?;
        println!("Created {path}");
    }

    Ok(())
}

fn load_local(config: &AppConfig) -> Result<local_config::LocalConfig, Box<dyn Error>> {
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

    Ok(local)
}

