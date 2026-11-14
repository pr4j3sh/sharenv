use std::{error::Error, fs, io, path::Path};

use serde::{Deserialize, Serialize};

use crate::settings::AppConfig;
use crate::utils::fs_secure;

pub const CREATED_BY_INIT: &str = "init";
/// Provenance marker for a binding recorded by `invites accept`.
pub const CREATED_BY_INVITES: &str = "invites-accept";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LocalConfig {
    #[serde(default)]
    pub project: ProjectSection,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProjectSection {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub owner_id: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub created_by: String,
}

pub fn save(config: &AppConfig, project_id: &str) -> Result<(), Box<dyn Error>> {
    let mut local = load(config).unwrap_or_default();
    local.project.id = project_id.to_string();

    write(config, &local)
}

/// Load the local project config and return the current project id, or fail
/// with the standard message pointing at `init`.
pub fn project_id(config: &AppConfig) -> Result<String, Box<dyn Error>> {
    let local = load(config).map_err(|error| {
        format!(
            "failed to load {}: {error}",
            config.config_file_path
        )
    })?;

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

/// Store the project identity recorded by `init` (or `invites accept`).
/// These fields are advisory: they detect drift, but a hand-written config
/// can forge them.
pub fn save_identity(
    config: &AppConfig,
    project_id: &str,
    name: &str,
    owner_id: Option<&str>,
    created_by: &str,
) -> Result<(), Box<dyn Error>> {
    let mut local = load(config).unwrap_or_default();

    local.project.id = project_id.to_string();
    local.project.name = name.to_string();
    local.project.owner_id = owner_id.unwrap_or_default().to_string();
    local.project.base_url = config.api_config.base_url.trim_end_matches('/').to_string();
    local.project.created_by = created_by.to_string();

    write(config, &local)
}

fn write(config: &AppConfig, local: &LocalConfig) -> Result<(), Box<dyn Error>> {
    fs_secure::path_inside_cwd(Path::new(&config.config_file_path))?;
    fs_secure::ensure_secure_dir(Path::new(&config.config_dir_name))?;

    let path = Path::new(&config.config_file_path);
    fs_secure::atomic_write_secure(path, &toml::to_string(local)?)?;

    Ok(())
}

pub fn load(config: &AppConfig) -> Result<LocalConfig, Box<dyn Error>> {
    let content = fs_secure::safe_read(Path::new(&config.config_file_path))?;

    Ok(toml::from_str(&content)?)
}

/// Remove the local project config, ignoring an already-missing file.
/// A symlink at the target is refused.
pub fn remove(config: &AppConfig) -> Result<(), Box<dyn Error>> {
    let path = Path::new(&config.config_file_path);

    match fs::symlink_metadata(path) {
        Ok(meta) => {
            if meta.file_type().is_symlink() {
                return Err(format!(
                    "refusing to remove '{}': it is a symlink",
                    config.config_file_path
                )
                .into());
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    }

    fs::remove_file(path)?;

    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir(parent);
    }

    Ok(())
}
