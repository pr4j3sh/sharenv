use std::error::Error;

use crate::{
    settings::AppConfig,
    utils::{
        args::{FlagSpec, parse_flags, parse_switch},
        local_config,
        output::sanitize,
        prompt, session,
        validation::{validate_environment_name, validate_project_name},
    },
};

pub async fn run(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    let specs = [
        FlagSpec {
            key: "project",
            names: &["--project", "-p"],
            label: "project name",
        },
        FlagSpec {
            key: "environment",
            names: &["--environment", "-e"],
            label: "environment name",
        },
    ];

    let (args, yes) = parse_switch(args, &["--yes", "-y"]);
    let mut flags = parse_flags(&args, &specs)?;

    let project_name = flags.remove("project");
    let environment_name = flags.remove("environment");

    match (project_name, environment_name) {
        (Some(_), Some(_)) => {
            Err("specify either -p/--project or -e/--environment, not both".into())
        }
        (None, None) => delete_current_project(config, yes).await,
        (Some(name), None) => delete_project_by_name(config, &name, yes).await,
        (None, Some(name)) => delete_environment(config, &name, yes).await,
    }
}

fn prompt_password() -> Result<String, Box<dyn Error>> {
    let password = rpassword::prompt_password("your password (required by the server): ")?;

    if password.is_empty() {
        return Err("password cannot be empty".into());
    }

    Ok(password)
}

async fn delete_current_project(config: &AppConfig, yes: bool) -> Result<(), Box<dyn Error>> {
    let project_id = load_project_id(config)?;

    if !yes
        && !prompt::confirm(&format!(
            "Delete current project ({}) and all its environments/variables? [y/N]: ",
            sanitize(&project_id)
        ))?
    {
        println!("Cancelled.");
        return Ok(());
    }

    let password = prompt_password()?;
    let client = session::authenticated_client(config).await?;
    let deleted = client.delete_project(&project_id, &password).await?;

    local_config::remove(config).map_err(|error| {
        format!(
            "project deleted remotely, but failed to remove {}: {error}",
            config.config_file_path
        )
    })?;

    println!(
        "Deleted current project ({}).",
        sanitize(&deleted.project_id)
    );

    Ok(())
}

async fn delete_project_by_name(
    config: &AppConfig,
    project_name: &str,
    yes: bool,
) -> Result<(), Box<dyn Error>> {
    validate_project_name(project_name)?;

    if !yes
        && !prompt::confirm(&format!(
            "Delete project '{}' and all its environments/variables? [y/N]: ",
            sanitize(project_name)
        ))?
    {
        println!("Cancelled.");
        return Ok(());
    }

    let password = prompt_password()?;
    let client = session::authenticated_client(config).await?;
    let deleted = client.delete_project_by_name(project_name, &password).await?;

    let local_id = local_config::load(config)
        .ok()
        .map(|local| local.project.id)
        .filter(|id| !id.is_empty());

    if local_id.as_deref() == Some(deleted.project_id.as_str()) {
        local_config::remove(config).map_err(|error| {
            format!(
                "project deleted remotely, but failed to remove {}: {error}",
                config.config_file_path
            )
        })?;
    }

    println!(
        "Deleted project '{}' ({}).",
        sanitize(project_name),
        sanitize(&deleted.project_id)
    );

    Ok(())
}

async fn delete_environment(
    config: &AppConfig,
    environment_name: &str,
    yes: bool,
) -> Result<(), Box<dyn Error>> {
    validate_environment_name(environment_name)?;

    let project_id = load_project_id(config)?;
    let client = session::authenticated_client(config).await?;
    let environments = client
        .get_environments(&project_id, Some(environment_name))
        .await?;
    let environment = environments
        .into_iter()
        .find(|environment| environment.name == environment_name)
        .ok_or_else(|| format!("environment '{environment_name}' not found"))?;

    if !yes
        && !prompt::confirm(&format!(
            "Delete environment '{}' ({}) and all its variables? [y/N]: ",
            sanitize(environment_name),
            sanitize(&environment._id)
        ))?
    {
        println!("Cancelled.");
        return Ok(());
    }

    let deleted = client.delete_environment(&environment._id).await?;

    println!(
        "Deleted environment '{}' ({}).",
        sanitize(environment_name),
        sanitize(&deleted.environment_id)
    );

    Ok(())
}

fn load_project_id(config: &AppConfig) -> Result<String, Box<dyn Error>> {
    let local =
        local_config::load(config).map_err(|_| format!("no {} found", config.config_file_path))?;

    if local.project.id.is_empty() {
        return Err(format!(
            "project id is missing from {}; run '{} init' first",
            config.config_file_path,
            env!("CARGO_PKG_NAME")
        )
        .into());
    }

    Ok(local.project.id)
}
