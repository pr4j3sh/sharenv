use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

pub(super) fn ensure_config_exists() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dirs =
        ProjectDirs::from("", "", "sharenv").ok_or("Could not determine config directory")?;

    let config_dir = dirs.config_dir();
    let config_path = config_dir.join("config.toml");

    if !config_path.exists() {
        fs::create_dir_all(config_dir)?;

        fs::write(&config_path, include_str!("../../config.toml"))?;
    }

    Ok(config_path)
}
