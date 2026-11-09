use std::error::Error;

use crate::{
    api::client::ApiClient,
    settings::AppConfig,
    utils::{local_config, output::sanitize, token},
};

pub async fn run(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    let subject = args.first().map(String::as_str).unwrap_or("");

    match subject {
        "projects" | "project" | "p" => list_projects(config).await,
        "environments" | "environment" | "env" | "e" => list_environments(config).await,
        "" => Err(format!(
            "missing list subject; usage: {} list <projects|environments>",
            env!("CARGO_PKG_NAME")
        )
        .into()),
        other => Err(format!(
            "'{other}' is not a valid list subject; use 'projects' or 'environments'"
        )
        .into()),
    }
}

async fn list_projects(config: &AppConfig) -> Result<(), Box<dyn Error>> {
    let client = client(config)?;
    let projects = client.get_projects().await?;

    if projects.is_empty() {
        println!("No projects found.");
        return Ok(());
    }

    let rows = projects
        .into_iter()
        .map(|project| (sanitize(&project._id), sanitize(&project.name)))
        .collect::<Vec<_>>();

    print_table("ID", "NAME", &rows);

    Ok(())
}

async fn list_environments(config: &AppConfig) -> Result<(), Box<dyn Error>> {
    let local =
        local_config::load(config).map_err(|_| format!("no {} found", config.config_file_path))?;

    if local.project.id.is_empty() {
        return Err(format!("project id is missing from {}", config.config_file_path).into());
    }

    let client = client(config)?;
    let environments = client.get_environments(&local.project.id, None).await?;

    if environments.is_empty() {
        println!("No environments found.");
        return Ok(());
    }

    let rows = environments
        .into_iter()
        .map(|environment| (sanitize(&environment._id), sanitize(&environment.name)))
        .collect::<Vec<_>>();

    print_table("ID", "NAME", &rows);

    Ok(())
}

fn client(config: &AppConfig) -> Result<ApiClient, Box<dyn Error>> {
    let token = token::get_token(&config.auth)?.ok_or_else(|| {
        format!(
            "not logged in; run '{} login' first",
            env!("CARGO_PKG_NAME")
        )
    })?;

    Ok(ApiClient::new(&config.api_config)?.with_token(token))
}

fn print_table(first_header: &str, second_header: &str, rows: &[(String, String)]) {
    let first_width = rows
        .iter()
        .map(|(first, _)| first.len())
        .chain(std::iter::once(first_header.len()))
        .max()
        .unwrap_or(first_header.len());

    println!("{first_header:<first_width$}  {second_header}");

    for (first, second) in rows {
        println!("{first:<first_width$}  {second}");
    }
}
