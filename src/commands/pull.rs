use std::{collections::HashMap, error::Error, path::Path};

use crate::{
    api::client::ApiClient,
    models::EnvironmentModel,
    settings::AppConfig,
    utils::{
        args::parse_switch,
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
    let mut env_name: Option<String> = None;
    let mut project_id: Option<String> = None;
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--project-id" => {
                let value = iter.next().ok_or("missing value for --project-id")?;

                if value.starts_with('-') {
                    return Err("missing value for --project-id".into());
                }

                project_id = Some(value.clone());
            }
            value if value.starts_with("--project-id=") => {
                let value = &value["--project-id=".len()..];

                if value.is_empty() {
                    return Err("missing value for --project-id".into());
                }

                project_id = Some(value.to_string());
            }
            value if value.starts_with('-') => {
                return Err(format!("unexpected argument '{value}'").into());
            }
            value => {
                if env_name.is_some() {
                    return Err(format!("unexpected argument '{value}'").into());
                }

                env_name = Some(value.to_string());
            }
        }
    }

    if let Some(env_name) = &env_name {
        validate_environment_name(env_name)?;
    }

    let project_id = match project_id {
        Some(project_id) => project_id,
        None => {
            let local = local_config::load(config)
                .map_err(|_| format!("no {} found", config.config_file_path))?;

            if local.project.id.is_empty() {
                return Err(format!(
                    "project id is missing from {}; pass --project-id <project_id>",
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
        read_env_file(&path)?
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
