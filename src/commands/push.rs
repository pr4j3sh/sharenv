use std::{collections::HashMap, error::Error, path::Path};

use serde::Serialize;

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
        output,
        output::sanitize,
        project_guard::{self, Access},
        prompt, session,
        validation::validate_environment_name,
    },
};

#[derive(Serialize, Default)]
struct EnvPushSummary {
    environment: String,
    created: Vec<String>,
    updated: Vec<String>,
    unchanged: Vec<String>,
    skipped: Vec<String>,
    kept: Vec<String>,
    undecryptable: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<HashMap<String, String>>,
}

pub async fn run(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    let json = config.json_output;
    let (args, show_values) = parse_switch(args, &["--show-values"]);
    let env_name = take_positional(&args)?;

    if let Some(env_name) = &env_name {
        validate_environment_name(env_name)?;
    }

    let local = local_config::load(config)
        .map_err(|error| format!("failed to load {}: {error}", config.config_file_path))?;

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

    let summaries = match env_name {
        Some(env_name) => {
            let environments = client
                .get_environments(&local.project.id, Some(&env_name))
                .await?;
            let environment = environments
                .into_iter()
                .find(|environment| environment.name == env_name)
                .ok_or_else(|| format!("environment '{env_name}' not found"))?;

            vec![push_environment(config, &client, &environment, show_values).await?]
        }
        None => push_all_environments(config, &client, &local.project.id, show_values).await?,
    };

    if json {
        output::json(&serde_json::json!({ "environments": summaries }));
    }

    Ok(())
}

async fn push_all_environments(
    config: &AppConfig,
    client: &ApiClient,
    project_id: &str,
    show_values: bool,
) -> Result<Vec<EnvPushSummary>, Box<dyn Error>> {
    let json = config.json_output;
    let environments = client.get_environments(project_id, None).await?;

    if environments.is_empty() {
        output::line(json, "No environments found.");
        return Ok(Vec::new());
    }

    let names = environments
        .iter()
        .map(|environment| sanitize(&environment.name))
        .collect::<Vec<_>>()
        .join(", ");

    if !prompt::confirm(&format!("Push all environments ({names})? [y/N]: "))? {
        output::line(json, "Cancelled.");
        return Ok(Vec::new());
    }

    let mut summaries = Vec::with_capacity(environments.len());

    for environment in environments {
        let path = env_file_path(config, &environment.name);

        if !Path::new(&path).exists() {
            output::line(json, format!("Skipping '{}': {path} not found", sanitize(&environment.name)));
            continue;
        }

        output::line(json, format!("Environment '{}':", sanitize(&environment.name)));
        summaries.push(push_environment(config, client, &environment, show_values).await?);
    }

    Ok(summaries)
}

async fn push_environment(
    config: &AppConfig,
    client: &ApiClient,
    environment: &EnvironmentModel,
    show_values: bool,
) -> Result<EnvPushSummary, Box<dyn Error>> {
    let json = config.json_output;
    let env_display = sanitize(&environment.name);
    let path = env_file_path(config, &environment.name);
    let variables = read_env_file(&path)?;

    if variables.is_empty() {
        output::line(json, format!("No variables found in {path}"));

        let mut summary = EnvPushSummary {
            environment: environment.name.clone(),
            ..EnvPushSummary::default()
        };

        if show_values {
            summary.values = Some(HashMap::new());
        }

        return Ok(summary);
    }

    let remote = client.get_variables(&environment._id).await?;
    let (remote, undecryptable) = filter_decryptable(remote);
    let mut remote_by_name = remote
        .into_iter()
        .map(|variable| (variable.name.clone(), variable))
        .collect::<HashMap<_, _>>();

    let mut summary = EnvPushSummary {
        environment: environment.name.clone(),
        undecryptable: undecryptable
            .into_iter()
            .map(|variable| variable.name)
            .collect(),
        ..EnvPushSummary::default()
    };

    let mut values = show_values.then(HashMap::new);

    for variable in variables {
        let name = sanitize(&variable.name);

        if let Some(map) = values.as_mut() {
            map.insert(variable.name.clone(), variable.value.clone());
        }

        match remote_by_name.remove(&variable.name) {
            None => match client
                .create_variable(&environment._id, variable.name.clone(), variable.value)
                .await
            {
                Ok(_) => {
                    summary.created.push(variable.name.clone());
                    output::line(json, format!("+ {name}"));
                }
                Err(error) if is_conflict_error(error.as_ref()) => {
                    summary.skipped.push(variable.name.clone());
                    output::line(json, format!("= {name} already exists, skipped"));
                }
                Err(error) => {
                    return Err(format!("failed to push '{name}': {error}").into());
                }
            },
            Some(remote) if remote.value.as_deref() == Some(variable.value.as_str()) => {
                summary.unchanged.push(variable.name.clone());
                output::line(json, format!("= {name} unchanged"));
            }
            Some(remote) => {
                output::line(json, format!("{name} changed:"));

                if show_values {
                    output::line(
                        json,
                        format!(
                            "  remote: {}",
                            sanitize(remote.value.as_deref().unwrap_or("<undecryptable>"))
                        ),
                    );
                    output::line(json, format!("  local:  {}", sanitize(&variable.value)));
                }

                if prompt::confirm(&format!("Update remote value for '{name}'? [y/N]: "))? {
                    client
                        .update_variable(&remote._id, variable.value.clone())
                        .await
                        .map_err(|error| format!("failed to update '{name}': {error}"))?;
                    summary.updated.push(variable.name.clone());
                    output::line(json, format!("~ {name} updated"));
                } else {
                    summary.kept.push(variable.name.clone());
                    output::line(json, format!("= {name} kept remote value"));
                }
            }
        }
    }

    summary.values = values;

    output::line(
        json,
        format!(
            "Pushed to '{env_display}': {} created, {} updated, {} unchanged, {} skipped, {} kept",
            summary.created.len(),
            summary.updated.len(),
            summary.unchanged.len(),
            summary.skipped.len(),
            summary.kept.len()
        ),
    );

    Ok(summary)
}
