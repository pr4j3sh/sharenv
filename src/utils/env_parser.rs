use crate::models::EnvironmentVariable;
use std::{error::Error, fs};

pub fn read_env_file(path: &str) -> Result<Vec<EnvironmentVariable>, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;

    if content.trim().is_empty() {
        return Err(format!("{path} is empty").into());
    }

    let mut variables = Vec::new();

    for (index, line) in content.lines().enumerate() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (name, value) = line
            .split_once('=')
            .ok_or_else(|| format!("Invalid entry in {path} at line {}", index + 1))?;

        let name = name.trim();

        if name.is_empty() {
            return Err(format!("Empty variable name in {path} at line {}", index + 1).into());
        }

        variables.push(EnvironmentVariable {
            name: name.to_string(),
            value: value.trim().to_string(),
        });
    }

    if variables.is_empty() {
        return Err(format!("{path} contains no environment variables").into());
    }

    Ok(variables)
}
