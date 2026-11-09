use std::{collections::HashMap, error::Error, path::Path};

use crate::{
    api::client::ApiClient,
    models::EnvironmentModel,
    settings::AppConfig,
    utils::{
        args::parse_value_arg, env_file::env_file_path, env_parser::read_env_file,
        error::is_conflict_error, local_config, prompt, token,
        validation::validate_environment_name,
    },
};

pub async fn run(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    let env_name = parse_value_arg(args, "--environment", "-e", "environment name")?;

    if let Some(env_name) = &env_name {
        validate_environment_name(env_name)?;
    }

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

    match env_name {
        Some(env_name) => {
            let environments = client
                .get_environments(&local.project.id, Some(&env_name))
                .await?;
            let environment = environments
                .into_iter()
                .find(|environment| environment.name == env_name)
                .ok_or_else(|| format!("environment '{env_name}' not found"))?;

            push_environment(config, &client, &environment).await
        }
        None => push_all_environments(config, &client, &local.project.id).await,
    }
}

async fn push_all_environments(
    config: &AppConfig,
    client: &ApiClient,
    project_id: &str,
) -> Result<(), Box<dyn Error>> {
    let environments = client.get_environments(project_id, None).await?;

    if environments.is_empty() {
        println!("No environments found.");
        return Ok(());
    }

    let names = environments
        .iter()
        .map(|environment| environment.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    if !prompt::confirm(&format!("Push all environments ({names})? [y/N]: "))? {
        println!("Cancelled.");
        return Ok(());
    }

    for environment in environments {
        let path = env_file_path(config, &environment.name);

        if !Path::new(&path).exists() {
            println!("Skipping '{}': {path} not found", environment.name);
            continue;
        }

        println!("Environment '{}':", environment.name);
        push_environment(config, client, &environment).await?;
    }

    Ok(())
}

async fn push_environment(
    config: &AppConfig,
    client: &ApiClient,
    environment: &EnvironmentModel,
) -> Result<(), Box<dyn Error>> {
    let path = env_file_path(config, &environment.name);
    let variables =
        read_env_file(&path).map_err(|error| format!("failed to read {path}: {error}"))?;

    if variables.is_empty() {
        println!("No variables found in {path}");
        return Ok(());
    }

    let remote = client.get_variables(&environment._id).await?;
    let mut remote_by_name = remote
        .into_iter()
        .map(|variable| (variable.name.clone(), variable))
        .collect::<HashMap<_, _>>();

    let mut created = 0;
    let mut updated = 0;
    let mut unchanged = 0;
    let mut skipped = 0;
    let mut kept = 0;

    for variable in variables {
        match remote_by_name.remove(&variable.name) {
            None => match client
                .create_variable(&environment._id, variable.name.clone(), variable.value)
                .await
            {
                Ok(_) => {
                    created += 1;
                    println!("+ {}", variable.name);
                }
                Err(error) if is_conflict_error(error.as_ref()) => {
                    skipped += 1;
                    println!("= {} already exists, skipped", variable.name);
                }
                Err(error) => {
                    return Err(format!("failed to push '{}': {error}", variable.name).into());
                }
            },
            Some(remote) if remote.value == variable.value => {
                unchanged += 1;
                println!("= {} unchanged", variable.name);
            }
            Some(remote) => {
                println!("{} changed:", variable.name);
                println!("  remote: {}", remote.value);
                println!("  local:  {}", variable.value);

                if prompt::confirm(&format!(
                    "Update remote value for '{}'? [y/N]: ",
                    variable.name
                ))? {
                    client
                        .update_variable(&remote._id, variable.value.clone())
                        .await
                        .map_err(|error| {
                            format!("failed to update '{}': {error}", variable.name)
                        })?;
                    updated += 1;
                    println!("~ {} updated", variable.name);
                } else {
                    kept += 1;
                    println!("= {} kept remote value", variable.name);
                }
            }
        }
    }

    println!(
        "Pushed to '{}': {created} created, {updated} updated, \
         {unchanged} unchanged, {skipped} skipped, {kept} kept",
        environment.name
    );

    Ok(())
}
