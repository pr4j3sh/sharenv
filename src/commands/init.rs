use std::env;

use crate::{
    settings::AppConfig,
    utils::{
        args::take_positional,
        env_file::{create_env_file, env_file_path},
        local_config,
        output::sanitize,
        session,
        validation::validate_project_name,
    },
};

pub async fn run(
    config: &AppConfig,
    args: Vec<String>,
    adopt: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = take_positional(&args)?;
    let path = env::current_dir()?;
    let cwd_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let project_name = name.unwrap_or(cwd_name);

    let client = session::authenticated_client(config).await?;
    let current_user = session::current_user(&client);

    let local = local_config::load(config).ok();
    let local_id = local
        .as_ref()
        .map(|local| local.project.id.as_str())
        .filter(|id| !id.is_empty());

    if adopt {
        let local_id = local_id.ok_or_else(|| {
            format!(
                "no initialized project found in {}; run '{} init' first",
                config.config_file_path,
                env!("CARGO_PKG_NAME")
            )
        })?;

        let projects = client.get_projects().await?;
        let project = projects
            .into_iter()
            .find(|project| project._id == local_id)
            .ok_or_else(|| {
                format!(
                    "cannot adopt project '{}': you are not a member or it was deleted",
                    sanitize(local_id)
                )
            })?;

        local_config::save_identity(config, &project._id, &project.name, Some(&project.user_id))?;
        println!(
            "Recorded project identity for '{}' (owner: {}) ({})",
            sanitize(&project.name),
            sanitize(&project.user_id),
            project._id
        );
        return Ok(());
    }

    if let Some(local_id) = local_id {
        let projects = client.get_projects().await?;

        if let Some(project) = projects.into_iter().find(|project| project._id == local_id) {
            println!(
                "Project '{}' is already initialized ({})",
                sanitize(&project.name),
                project._id
            );
            println!(
                "To trust this local config, run '{} init --adopt'",
                env!("CARGO_PKG_NAME")
            );
            return Ok(());
        }
    }

    validate_project_name(&project_name)?;

    if local.is_none() {
        local_config::save(config, "")?;
    }

    let project = client.create_project(project_name.clone()).await?;
    local_config::save_identity(
        config,
        &project.project_id,
        &project_name,
        current_user.as_deref(),
    )?;

    println!(
        "Created project '{}' ({})",
        sanitize(&project_name),
        project.project_id
    );

    for env in &config.environment_files.environments {
        create_env_file(&env_file_path(config, env))?;

        let environment = client
            .create_environment(&project.project_id, env.clone())
            .await?;

        println!(
            "Created environment '{env}' ({})",
            environment.environment_id
        );
    }

    println!(
        "Project '{}' initialized successfully!",
        sanitize(&project_name)
    );

    Ok(())
}
