use std::env;

use crate::{
    api::client::ApiClient,
    settings::AppConfig,
    utils::{
        env_file::{create_env_file, env_file_path},
        local_config, token,
        validation::validate_project_name,
    },
};

pub async fn run(
    config: &AppConfig,
    name: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = env::current_dir()?;
    let cwd_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let project_name = name.unwrap_or(cwd_name);
    validate_project_name(&project_name)?;

    let token = token::get_token(&config.auth)?.ok_or_else(|| {
        format!(
            "not logged in; run '{} login' first",
            env!("CARGO_PKG_NAME")
        )
    })?;
    let client = ApiClient::new(&config.api_config).with_token(token);

    let local = local_config::load(config).ok();

    if let Some(local_id) = local
        .as_ref()
        .map(|local| local.project.id.as_str())
        .filter(|id| !id.is_empty())
    {
        let projects = client.get_projects().await?;

        if let Some(project) = projects.into_iter().find(|project| project._id == local_id) {
            println!(
                "Project '{}' is already initialized ({})",
                project.name, project._id
            );
            return Ok(());
        }
    }

    if local.is_none() {
        local_config::save(config, "")?;
    }

    let project = client.create_project(project_name.clone()).await?;
    local_config::save(config, &project.project_id)?;

    println!(
        "Created project '{}' ({})",
        project_name, project.project_id
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

    println!("Project '{project_name}' initialized successfully!");

    Ok(())
}
