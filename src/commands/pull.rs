use std::{collections::HashMap, error::Error, path::Path};

use crate::{
    api::client::ApiClient,
    models::EnvironmentModel,
    settings::AppConfig,
    utils::{
        args::{FlagSpec, parse_flags, parse_switch},
        env_changes::{ChangePlan, filter_decryptable, plan_changes},
        env_file::{env_file_path, write_env_changes},
        env_parser::read_env_file,
        local_config,
        output::sanitize,
        project_guard::{self, Access},
        prompt, session,
        validation::validate_environment_name,
    },
};

pub async fn run(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    let (args, show_values) = parse_switch(args, &["--show-values"]);

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

    let mut flags = parse_flags(&args, &specs)?;

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

    let client = session::authenticated_client(config).await?;
    let current_user = session::current_user(&client);

    // `GET /project/` only lists owned or shared projects, so this refuses
    // non-members before any variable is fetched.
    if project_guard::authorize(
        config,
        &client,
        current_user.as_deref(),
        &project_id,
        Access::Pull,
    )
    .await?
    .is_none()
    {
        return Ok(());
    }

    match env_name {
        Some(env_name) => {
            let environments = client
                .get_environments(&project_id, Some(&env_name))
                .await?;
            let environment = environments
                .into_iter()
                .find(|environment| environment.name == env_name)
                .ok_or_else(|| format!("environment '{env_name}' not found"))?;

            pull_environment(config, &client, &environment, show_values).await
        }
        None => pull_all_environments(config, &client, &project_id, show_values).await,
    }
}

async fn pull_all_environments(
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

    if !prompt::confirm(&format!("Pull all environments ({names})? [y/N]: "))? {
        println!("Cancelled.");
        return Ok(());
    }

    for environment in environments {
        println!("Environment '{}':", sanitize(&environment.name));
        pull_environment(config, client, &environment, show_values).await?;
    }

    Ok(())
}

async fn pull_environment(
    config: &AppConfig,
    client: &ApiClient,
    environment: &EnvironmentModel,
    show_values: bool,
) -> Result<(), Box<dyn Error>> {
    let env_name = sanitize(&environment.name);
    let remote = client.get_variables(&environment._id).await?;
    let remote = filter_decryptable(remote);

    let path = env_file_path(config, &environment.name);
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
            let name = sanitize(&variable.name);

            if show_values {
                println!("  + {name}={}", sanitize(&variable.value));
            } else {
                println!("  + {name}");
            }
        }

        if prompt::confirm("Add these variables? [y/N]: ")? {
            additions = new_variables;
        }
    }

    let mut updates = HashMap::new();
    let mut kept = 0;

    for change in changed_variables {
        let name = sanitize(&change.name);
        println!("{name} changed:");

        if show_values {
            println!("  current: {}", sanitize(&change.current));
            println!("  remote:  {}", sanitize(&change.remote));
        }

        if prompt::confirm(&format!("Replace value for '{name}'? [y/N]: "))? {
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
