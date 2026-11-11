use std::{
    collections::HashMap,
    io,
    path::Path,
};

use crate::{
    settings::AppConfig,
    utils::{
        env_parser::{EnvVariable, format_line, parse_line},
        fs_secure,
        prompt,
    },
};

pub fn env_file_path(config: &AppConfig, env_name: &str) -> String {
    format!("{}.{env_name}", config.environment_files.env_file_prefix)
}

pub fn create_env_file(path: &str) -> io::Result<()> {
    let path_ref = Path::new(path);

    if path_ref.exists() {
        fs_secure::check_secret_file(path_ref)?;

        if prompt::confirm(&format!("{path} file already exists. Replace it? [y/N]: "))? {
            fs_secure::path_inside_cwd(path_ref)?;
            fs_secure::atomic_write_secure(path_ref, "")?;
            println!("Replaced {path} file");
        } else {
            println!("Keeping existing {path} file");
        }
    } else {
        fs_secure::atomic_write_secure(path_ref, "")?;
        println!("Created {path} file");
    }

    Ok(())
}

/// Apply accepted updates in place and append new variables, keeping comments
/// and existing formatting intact. Creates the file when it does not exist.
///
/// The result is a `0600` regular file; a symlink at `path` is refused.
pub fn write_env_changes(
    path: &str,
    updates: &HashMap<String, String>,
    additions: &[EnvVariable],
) -> io::Result<()> {
    let path_ref = Path::new(path);
    fs_secure::path_inside_cwd(path_ref)?;

    let content = match fs_secure::safe_read_opt(path_ref, true)? {
        Some(content) => content,
        None => String::new(),
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

    fs_secure::atomic_write_secure(path_ref, &output)
}
