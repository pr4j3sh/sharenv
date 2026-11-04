use std::{
    collections::HashMap,
    fs::{self, File},
    io,
    path::Path,
};

use crate::{
    settings::AppConfig,
    utils::{
        env_parser::{EnvVariable, format_line, parse_line},
        prompt,
    },
};

pub fn env_file_path(config: &AppConfig, env_name: &str) -> String {
    format!("{}.{env_name}", config.environment_files.env_file_prefix)
}

pub fn create_env_file(path: &str) -> io::Result<()> {
    if Path::new(path).exists() {
        if prompt::confirm(&format!("{path} already exists. Replace it? [y/N]: "))? {
            File::create(path)?;
            println!("Replaced {path}");
        } else {
            println!("Keeping existing {path}");
        }
    } else {
        File::create(path)?;
        println!("Created {path}");
    }

    Ok(())
}

/// Apply accepted updates in place and append new variables, keeping comments
/// and existing formatting intact. Creates the file when it does not exist.
pub fn write_env_changes(
    path: &str,
    updates: &HashMap<String, String>,
    additions: &[EnvVariable],
) -> io::Result<()> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error),
    };

    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();

    for line in &mut lines {
        let variable = match parse_line(line.as_str()) {
            Some(Ok(variable)) => variable,
            _ => continue,
        };

        if let Some(value) = updates.get(&variable.name) {
            *line = format_line(&EnvVariable {
                name: variable.name,
                value: value.clone(),
            });
        }
    }

    for variable in additions {
        lines.push(format_line(variable));
    }

    let mut output = lines.join("\n");

    if !output.is_empty() {
        output.push('\n');
    }

    fs::write(path, output)
}
