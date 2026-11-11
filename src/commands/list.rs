use std::error::Error;

use crate::{
    settings::AppConfig,
    utils::{local_config, output::sanitize, session},
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
    let client = session::authenticated_client(config).await?;
    let projects = client.get_projects().await?;

    if projects.is_empty() {
        println!("No projects found.");
        return Ok(());
    }

    let rows = projects
        .into_iter()
        .map(|project| {
            vec![
                sanitize(&project._id),
                sanitize(&project.name),
                sanitize(&project.role),
            ]
        })
        .collect::<Vec<_>>();

    print_table(&["ID", "NAME", "ROLE"], &rows);

    Ok(())
}

async fn list_environments(config: &AppConfig) -> Result<(), Box<dyn Error>> {
    let local =
        local_config::load(config).map_err(|_| format!("no {} found", config.config_file_path))?;

    if local.project.id.is_empty() {
        return Err(format!("project id is missing from {}", config.config_file_path).into());
    }

    let client = session::authenticated_client(config).await?;
    let environments = client.get_environments(&local.project.id, None).await?;

    if environments.is_empty() {
        println!("No environments found.");
        return Ok(());
    }

    let rows = environments
        .into_iter()
        .map(|environment| vec![sanitize(&environment._id), sanitize(&environment.name)])
        .collect::<Vec<_>>();

    print_table(&["ID", "NAME"], &rows);

    Ok(())
}

fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    let columns = headers.len();

    let widths = (0..columns)
        .map(|column| {
            rows.iter()
                .map(|row| row[column].len())
                .chain(std::iter::once(headers[column].len()))
                .max()
                .unwrap_or(headers[column].len())
        })
        .collect::<Vec<_>>();

    let print_row = |cells: &[String], last_column: usize| {
        for (index, cell) in cells.iter().enumerate() {
            if index > 0 {
                print!("  ");
            }

            if index == last_column {
                print!("{cell}");
            } else {
                print!("{cell:<width$}", width = widths[index]);
            }
        }

        println!();
    };

    let header_row: Vec<String> = headers.iter().map(|header| header.to_string()).collect();
    print_row(&header_row, columns - 1);

    for row in rows {
        print_row(row, columns - 1);
    }
}
