//! Shared prompt helpers for choosing project environments.
//!
//! Used by `invite` (grant at invitation time) and `members environment` (change
//! grants later). Selection format: comma-separated environment names, or
//! `all` (the default on empty input).

use std::error::Error;

use crate::{models::EnvironmentModel, utils::output::sanitize, utils::prompt::prompt_line};

pub fn print_environment_list(environments: &[EnvironmentModel]) {
    println!("Environments in this project:");

    for (index, environment) in environments.iter().enumerate() {
        println!("  {}. {}", index + 1, sanitize(&environment.name));
    }
}

pub fn prompt_selection(
    environments: &[EnvironmentModel],
    label: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    let selection = prompt_line(label)?;

    pick_environments(&selection, environments)
}

pub fn pick_environments(
    selection: &str,
    environments: &[EnvironmentModel],
) -> Result<Vec<String>, Box<dyn Error>> {
    let trimmed = selection.trim();

    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("all") {
        return Ok(environments
            .iter()
            .map(|environment| environment._id.clone())
            .collect());
    }

    let mut ids = Vec::new();

    for name in trimmed.split(',').map(str::trim).filter(|name| !name.is_empty()) {
        let environment = environments
            .iter()
            .find(|environment| environment.name == name)
            .ok_or_else(|| {
                let valid = environments
                    .iter()
                    .map(|environment| environment.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("unknown environment '{name}'; valid: {valid}")
            })?;

        if !ids.contains(&environment._id) {
            ids.push(environment._id.clone());
        }
    }

    Ok(ids)
}

pub fn environment_names_for(
    ids: &[String],
    environments: &[EnvironmentModel],
) -> Vec<String> {
    if ids.is_empty() {
        return vec!["none".to_string()];
    }

    ids.iter()
        .map(|id| {
            environments
                .iter()
                .find(|environment| &environment._id == id)
                .map(|environment| environment.name.clone())
                .unwrap_or_else(|| id.clone())
        })
        .collect()
}
