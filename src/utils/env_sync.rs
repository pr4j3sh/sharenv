use mongodb::bson::Document;
use std::collections::HashSet;

use crate::models::EnvironmentVariable;

pub fn get_existing_names(
    project: &Document,
    env_name: &str,
) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let environments = project.get_array("environments")?;

    let environment = environments
        .iter()
        .filter_map(|item| item.as_document())
        .find(|item| item.get_str("name").ok() == Some(env_name))
        .ok_or_else(|| format!("Environment '{env_name}' not found"))?;

    let variables = environment.get_array("variables")?;

    Ok(variables
        .iter()
        .filter_map(|item| item.as_document())
        .filter_map(|item| item.get_str("name").ok())
        .map(String::from)
        .collect())
}

pub fn find_new_variables<'a>(
    variables: &'a [EnvironmentVariable],
    existing_names: &HashSet<String>,
) -> Vec<&'a EnvironmentVariable> {
    variables
        .iter()
        .filter(|variable| !existing_names.contains(&variable.name))
        .collect()
}
