use std::{collections::HashMap, error::Error, path::Path};

use serde::Serialize;

use crate::{
    api::client::ApiClient,
    models::EnvironmentModel,
    settings::AppConfig,
    utils::{
        args::{parse_switches},
        env_changes::{ChangePlan, filter_decryptable, plan_changes},
        env_file::{env_file_path, write_env_changes},
        env_parser::read_env_file,
        local_config,
        output,
        output::sanitize,
        project_guard::{self, Access},
        prompt, session,
        validation::validate_environment_name,
    },
};

#[derive(Serialize, Default)]
struct EnvPullSummary {
    environment: String,
    dry_run: bool,
    added: Vec<String>,
    updated: Vec<String>,
    kept: Vec<String>,
    undecryptable: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<HashMap<String, String>>,
}

pub async fn run(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    let json = config.json_output;

    let (args, show_values, dry_run) = parse_switches(args, &["--show-values"], &["--dry-run"]);
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
            let local = local_config::load(config).map_err(|error| {
                format!("failed to load {}: {error}", config.config_file_path)
            })?;

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

    let summaries = match env_name {
        Some(env_name) => {
            let environments = client
                .get_environments(&project_id, Some(&env_name))
                .await?;
            let environment = environments
                .into_iter()
                .find(|environment| environment.name == env_name)
                .ok_or_else(|| format!("environment '{env_name}' not found"))?;

            vec![pull_environment(config, &client, &environment, show_values, dry_run).await?]
        }
        None => pull_all_environments(config, &client, &project_id, show_values, dry_run).await?,
    };

    if json {
        output::json(&serde_json::json!({ "environments": summaries }));
    }

    Ok(())
}

async fn pull_all_environments(
    config: &AppConfig,
    client: &ApiClient,
    project_id: &str,
    show_values: bool,
    dry_run: bool,
) -> Result<Vec<EnvPullSummary>, Box<dyn Error>> {
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

    if !dry_run && !prompt::confirm(&format!("Pull all environments ({names})? [y/N]: "))? {
        output::line(json, "Cancelled.");
        return Ok(Vec::new());
    }

    let mut summaries = Vec::with_capacity(environments.len());

    for environment in environments {
        output::line(json, format!("Environment '{}':", sanitize(&environment.name)));
        summaries.push(pull_environment(config, client, &environment, show_values, dry_run).await?);
    }

    Ok(summaries)
}

async fn pull_environment(
    config: &AppConfig,
    client: &ApiClient,
    environment: &EnvironmentModel,
    show_values: bool,
    dry_run: bool,
) -> Result<EnvPullSummary, Box<dyn Error>> {
    let json = config.json_output;
    let env_name = sanitize(&environment.name);
    let remote = client.get_variables(&environment._id).await?;
    let (remote, undecryptable) = filter_decryptable(remote);

    let mut summary = EnvPullSummary {
        environment: environment.name.clone(),
        dry_run,
        undecryptable: undecryptable
            .into_iter()
            .map(|variable| variable.name)
            .collect(),
        ..EnvPullSummary::default()
    };

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
        output::line(json, format!("Already up to date for '{env_name}'"));
        return Ok(summary);
    }

    let mut values = show_values.then(HashMap::new);
    let mut additions = Vec::new();

    if !new_variables.is_empty() {
        output::line(json, "New variables:");
        for variable in &new_variables {
            let name = sanitize(&variable.name);

            if show_values {
                output::line(json, format!("  + {name}={}", sanitize(&variable.value)));
            } else {
                output::line(json, format!("  + {name}"));
            }
        }

        if dry_run || prompt::confirm("Add these variables? [y/N]: ")? {
            for variable in &new_variables {
                summary.added.push(variable.name.clone());
                if let Some(map) = values.as_mut() {
                    map.insert(variable.name.clone(), variable.value.clone());
                }
            }

            additions = new_variables;
        }
    }

    let mut updates = HashMap::new();

    for change in changed_variables {
        let name = sanitize(&change.name);
        output::line(json, format!("{name} changed:"));

        if show_values {
            output::line(json, format!("  current: {}", sanitize(&change.current)));
            output::line(json, format!("  remote:  {}", sanitize(&change.remote)));
        }

        if dry_run || prompt::confirm(&format!("Replace value for '{name}'? [y/N]: "))? {
            if let Some(map) = values.as_mut() {
                map.insert(change.name.clone(), change.remote.clone());
            }

            updates.insert(change.name, change.remote);
        } else {
            summary.kept.push(change.name);
        }
    }

    let added = additions.len();
    let updated = updates.len();

    for name in updates.keys() {
        summary.updated.push(name.clone());
    }

    if added == 0 && updated == 0 {
        output::line(json, format!("No changes applied to {path}"));
        summary.values = values;
        return Ok(summary);
    }

    if !dry_run {
        write_env_changes(&path, &updates, &additions)
            .map_err(|error| format!("failed to write {path}: {error}"))?;
    }

    let verb = if dry_run { "Would write" } else { "Pulled into" };
    output::line(
        json,
        format!("{verb} {path}: {added} added, {updated} updated, {} kept", summary.kept.len()),
    );

    summary.values = values;

    Ok(summary)
}
