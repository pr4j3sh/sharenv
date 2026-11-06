use std::{error::Error, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::settings::AppConfig;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LocalConfig {
    #[serde(default)]
    pub project: ProjectSection,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProjectSection {
    #[serde(default)]
    pub id: String,
}

pub fn save(config: &AppConfig, project_id: &str) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&config.config_dir_name)?;

    let path = Path::new(&config.config_file_path);

    let mut local = load(config).unwrap_or_default();
    local.project.id = project_id.to_string();

    fs::write(path, toml::to_string(&local)?)?;

    Ok(())
}

#[allow(dead_code)]
pub fn load(config: &AppConfig) -> Result<LocalConfig, Box<dyn Error>> {
    let content = fs::read_to_string(&config.config_file_path)?;

    Ok(toml::from_str(&content)?)
}
