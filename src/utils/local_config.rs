use std::{error::Error, fs, io, path::Path};

use serde::{Deserialize, Serialize};

use crate::settings::AppConfig;

pub const CREATED_BY_INIT: &str = "init";

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

/// Store the project identity recorded by `init`. These fields are advisory:
/// they detect drift, but a hand-written config can forge them.
pub fn save_identity(
    config: &AppConfig,
    project_id: &str,
    name: &str,
    owner_id: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let mut local = load(config).unwrap_or_default();

    local.project.id = project_id.to_string();
    local.project.name = name.to_string();
    local.project.owner_id = owner_id.unwrap_or_default().to_string();
    local.project.base_url = config.api_config.base_url.trim_end_matches('/').to_string();
    local.project.created_by = CREATED_BY_INIT.to_string();

    write(config, &local)
}

fn write(config: &AppConfig, local: &LocalConfig) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&config.config_dir_name)?;

    let path = Path::new(&config.config_file_path);
    fs::write(path, toml::to_string(local)?)?;

    Ok(())
}

#[allow(dead_code)]
pub fn load(config: &AppConfig) -> Result<LocalConfig, Box<dyn Error>> {
    let content = fs::read_to_string(&config.config_file_path)?;

    Ok(toml::from_str(&content)?)
}

/// Remove the local project config, ignoring an already-missing file.
pub fn remove(config: &AppConfig) -> Result<(), Box<dyn Error>> {
    let path = Path::new(&config.config_file_path);

    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    }

    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir(parent);
    }

    Ok(())
}
