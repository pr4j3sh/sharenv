use std::{collections::HashMap, error::Error, path::Path};

use crate::{
    api::client::ApiClient,
    models::EnvironmentModel,
    settings::AppConfig,
    utils::{
        args::{parse_switch, take_positional},
        env_changes::filter_decryptable,
        env_file::env_file_path,
        env_parser::read_env_file,
        error::is_conflict_error,
        local_config,
        output::sanitize,
        project_guard::{self, Access},
        prompt, session,
        validation::validate_environment_name,
    },
};

pub async fn run(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    let (args, show_values) = parse_switch(args, &["--show-values"]);
    let env_name = take_positional(&args)?;

    if let Some(env_name) = &env_name {
        validate_environment_name(env_name)?;
    }

    let local =
        local_config::load(config).map_err(|error| format!("failed to load {}: {error}", config.config_file_path))?;

    if local.project.id.is_empty() {
        return Err(format!("project id is missing from {}", config.config_file_path).into());
    }

    let client = session::authenticated_client(config).await?;
    let current_user = session::current_user(&client);

    if project_guard::authorize(
        config,
        &client,
        current_user.as_deref(),
        &local.project.id,
        Access::Push,
    )
    .await?
    .is_none()
    {
        return Ok(());
    }

    match env_name {
        Some(env_name) => {
            let environments = client
                .get_environments(&local.project.id, Some(&env_name))
                .await?;
            let environment = environments
                .into_iter()
                .find(|environment| environment.name == env_name)
                .ok_or_else(|| format!("environment '{env_name}' not found"))?;

            push_environment(config, &client, &environment, show_values).await
        }
        None => push_all_environments(config, &client, &local.project.id, show_values).await,
    }
}

async fn push_all_environments(
    config: &AppConfig,
    client: &ApiClient,
    project_id: &str,
    show_values: bool,
) -> Result<(), Box<dyn Error>> {
    let environments = client.get_environments(project_id, None).await?;

    if environments.is_empty() {
        println!("No environments found.");
        return Ok(());
    }

    let names = environments
        .iter()
        .map(|environment| sanitize(&environment.name))
        .collect::<Vec<_>>()
        .join(", ");

    if !prompt::confirm(&format!("Push all environments ({names})? [y/N]: "))? {
        println!("Cancelled.");
        return Ok(());
    }

    for environment in environments {
        let path = env_file_path(config, &environment.name);

        if !Path::new(&path).exists() {
            println!(
                "Skipping '{}': {path} not found",
                sanitize(&environment.name)
            );
            continue;
        }

        println!("Environment '{}':", sanitize(&environment.name));
        push_environment(config, client, &environment, show_values).await?;
    }

    Ok(())
}

async fn push_environment(
    config: &AppConfig,
    client: &ApiClient,
    environment: &EnvironmentModel,
    show_values: bool,
) -> Result<(), Box<dyn Error>> {
    let env_display = sanitize(&environment.name);
    let path = env_file_path(config, &environment.name);
    let variables =
        read_env_file(&path)?;

    if variables.is_empty() {
        println!("No variables found in {path}");
        return Ok(());
    }

    let remote = client.get_variables(&environment._id).await?;
    let remote = filter_decryptable(remote);
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
        let name = sanitize(&variable.name);

        match remote_by_name.remove(&variable.name) {
            None => match client
                .create_variable(&environment._id, variable.name.clone(), variable.value)
                .await
            {
                Ok(_) => {
                    created += 1;
                    println!("+ {name}");
                }
                Err(error) if is_conflict_error(error.as_ref()) => {
                    skipped += 1;
                    println!("= {name} already exists, skipped");
                }
                Err(error) => {
                    return Err(format!("failed to push '{name}': {error}").into());
                }
            },
            Some(remote) if remote.value.as_deref() == Some(variable.value.as_str()) => {
                unchanged += 1;
                println!("= {name} unchanged");
            }
            Some(remote) => {
                println!("{name} changed:");

                if show_values {
                    println!(
                        "  remote: {}",
                        sanitize(remote.value.as_deref().unwrap_or("<undecryptable>"))
                    );
                    println!("  local:  {}", sanitize(&variable.value));
                }

                if prompt::confirm(&format!("Update remote value for '{name}'? [y/N]: "))? {
                    client
                        .update_variable(&remote._id, variable.value.clone())
                        .await
                        .map_err(|error| format!("failed to update '{name}': {error}"))?;
                    updated += 1;
                    println!("~ {name} updated");
                } else {
                    kept += 1;
                    println!("= {name} kept remote value");
                }
            }
        }
    }

    println!(
        "Pushed to '{env_display}': {created} created, {updated} updated, \
         {unchanged} unchanged, {skipped} skipped, {kept} kept"
    );

    Ok(())
}
