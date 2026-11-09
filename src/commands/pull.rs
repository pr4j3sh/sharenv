use std::{collections::HashMap, error::Error, path::Path};

use crate::{
    api::client::ApiClient,
    models::EnvironmentModel,
    settings::AppConfig,
    utils::{
        args::{FlagSpec, parse_flags},
        env_changes::{ChangePlan, plan_changes},
        env_file::{env_file_path, write_env_changes},
        env_parser::read_env_file,
        local_config, prompt, token,
        validation::validate_environment_name,
    },
};

pub async fn run(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    let specs = [
        FlagSpec {
            key: "environment",
            names: &["--environment", "-e"],
            label: "environment name",
        },
        FlagSpec {
            key: "project_id",
            names: &["--pid", "--project_id", "-pid"],
            label: "project id",
        },
    ];

    let mut flags = parse_flags(args, &specs)?;

    let env_name = flags.remove("environment");

    if let Some(env_name) = &env_name {
        validate_environment_name(env_name)?;
    }

    let project_id = match flags.remove("project_id") {
        Some(project_id) => project_id,
        None => {
            let local = local_config::load(config)
                .map_err(|_| format!("no {} found", config.config_file_path))?;

            if local.project.id.is_empty() {
                return Err(format!(
                    "project id is missing from {}; pass --pid <project_id>",
                    config.config_file_path
                )
                .into());
            }

            local.project.id
        }
    };

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
                .get_environments(&project_id, Some(&env_name))
                .await?;
            let environment = environments
                .into_iter()
                .find(|environment| environment.name == env_name)
                .ok_or_else(|| format!("environment '{env_name}' not found"))?;

            pull_environment(config, &client, &environment).await
        }
        None => pull_all_environments(config, &client, &project_id).await,
    }
}

async fn pull_all_environments(
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

    if !prompt::confirm(&format!("Pull all environments ({names})? [y/N]: "))? {
        println!("Cancelled.");
        return Ok(());
    }

    for environment in environments {
        println!("Environment '{}':", environment.name);
        pull_environment(config, client, &environment).await?;
    }

    Ok(())
}

async fn pull_environment(
    config: &AppConfig,
    client: &ApiClient,
    environment: &EnvironmentModel,
) -> Result<(), Box<dyn Error>> {
    let env_name = &environment.name;
    let remote = client.get_variables(&environment._id).await?;

    let path = env_file_path(config, env_name);
    let existing = if Path::new(&path).exists() {
        read_env_file(&path).map_err(|error| format!("failed to read {path}: {error}"))?
    } else {
        Vec::new()
    };

    let ChangePlan {
        additions: new_variables,
        updates: changed_variables,
    } = plan_changes(&remote, &existing);

    if new_variables.is_empty() && changed_variables.is_empty() {
        println!("Already up to date for '{env_name}'");
        return Ok(());
    }

    let mut additions = Vec::new();

    if !new_variables.is_empty() {
        println!("New variables:");
        for variable in &new_variables {
            println!("  + {}={}", variable.name, variable.value);
        }

        if prompt::confirm("Add these variables? [y/N]: ")? {
            additions = new_variables;
        }
    }

    let mut updates = HashMap::new();
    let mut kept = 0;

    for change in changed_variables {
        println!("{} changed:", change.name);
        println!("  current: {}", change.current);
        println!("  remote:  {}", change.remote);

        if prompt::confirm(&format!("Replace value for '{}'? [y/N]: ", change.name))? {
            updates.insert(change.name, change.remote);
        } else {
            kept += 1;
        }
    }

    let added = additions.len();
    let updated = updates.len();

    if added == 0 && updated == 0 {
        println!("No changes applied to {path}");
        return Ok(());
    }

    write_env_changes(&path, &updates, &additions)
        .map_err(|error| format!("failed to write {path}: {error}"))?;

    println!("Pulled into {path}: {added} added, {updated} updated, {kept} kept");

    Ok(())
}
