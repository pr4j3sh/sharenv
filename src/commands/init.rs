use std::env;
use std::path::Path;

use crate::{
    api::client::ApiClient,
    models::ProjectModel,
    settings::AppConfig,
    utils::{
        args::take_positional,
        env_file::{create_env_file, env_file_path},
        error::is_conflict_error,
        fs_secure, local_config,
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

        local_config::save_identity(
            config,
            &project._id,
            &project.name,
            Some(&project.user_id),
            local_config::CREATED_BY_INIT,
        )?;
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
            return repair_initialized(config, &client, &project).await;
        }
    }

    validate_project_name(&project_name)?;
    preflight_env_files(config)?;

    if local.is_none() {
        local_config::save(config, "")?;
    }

    let project = client.create_project(project_name.clone()).await?;
    local_config::save_identity(
        config,
        &project.project_id,
        &project_name,
        current_user.as_deref(),
        local_config::CREATED_BY_INIT,
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

/// Fail before any remote call: existing env files must pass the secret-file
/// checks, and missing ones must resolve inside the working directory.
fn preflight_env_files(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    for env in &config.environment_files.environments {
        let path = env_file_path(config, env);
        let path_ref = Path::new(&path);

        if path_ref.exists() {
            fs_secure::check_secret_file(path_ref)?;
        } else {
            fs_secure::path_inside_cwd(path_ref)?;
        }
    }

    Ok(())
}

/// An "already initialized" project may be missing pieces from a partially
/// completed first run; create whatever is absent, idempotently.
async fn repair_initialized(
    config: &AppConfig,
    client: &ApiClient,
    project: &ProjectModel,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Project '{}' is already initialized ({})",
        sanitize(&project.name),
        project._id
    );

    let remote_environments = client.get_environments(&project._id, None).await?;

    let mut created_environments = Vec::new();
    let mut created_files = Vec::new();

    for env in &config.environment_files.environments {
        let path = env_file_path(config, env);
        let path_ref = Path::new(&path);

        if path_ref.exists() {
            fs_secure::check_secret_file(path_ref)?;
        } else {
            fs_secure::path_inside_cwd(path_ref)?;
            fs_secure::atomic_write_secure(path_ref, "")?;
            created_files.push(path.clone());
        }

        if remote_environments
            .iter()
            .any(|environment| environment.name == *env)
        {
            continue;
        }

        match client.create_environment(&project._id, env.clone()).await {
            Ok(environment) => {
                created_environments.push(format!("{env} ({})", environment.environment_id))
            }
            Err(error) if is_conflict_error(error.as_ref()) => {}
            Err(error) => {
                return Err(format!("failed to create environment '{env}': {error}").into());
            }
        }
    }

    for environment in &created_environments {
        println!("Created environment '{environment}'");
    }

    for file in &created_files {
        println!("Created {file} file");
    }

    if created_environments.is_empty() && created_files.is_empty() {
        println!("Everything is already in place; nothing to repair.");
        println!(
            "To trust this local config, run '{} init --adopt'",
            env!("CARGO_PKG_NAME")
        );
    } else {
        println!(
            "Restored the missing pieces; project '{}' is now fully initialized.",
            sanitize(&project.name)
        );
    }

    Ok(())
}
