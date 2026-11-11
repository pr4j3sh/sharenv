use std::fs;
use std::io::Write;
use std::path::Path;

use tempfile::NamedTempFile;

use super::version::CURRENT_CONFIG_VERSION;

pub(super) fn migrate_config(config_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(config_path)?;
    let mut config: toml::Value = toml::from_str(&content)?;

    let version = config
        .get("config_version")
        .and_then(toml::Value::as_integer)
        .unwrap_or(1);

    let mut version = u32::try_from(version)?;

    if version > CURRENT_CONFIG_VERSION {
        return Err(format!(
            "Config version {version} is newer than supported version \
             {CURRENT_CONFIG_VERSION}. Please upgrade Sharenv."
        )
        .into());
    }

    if version == CURRENT_CONFIG_VERSION {
        return Ok(());
    }

    while version < CURRENT_CONFIG_VERSION {
        match version {
            1 => {
                // v1 -> v2: add the new auth and invitation routes introduced
                // by the server-side security work.
                if let Some(routes) = config
                    .get_mut("api_config")
                    .and_then(|api| api.get_mut("routes"))
                    .and_then(|routes| routes.as_table_mut())
                {
                    let defaults = [
                        ("refresh", "/v1/auth/refresh"),
                        ("logout", "/v1/auth/logout"),
                        ("verify", "/v1/auth/verify"),
                        ("resend", "/v1/auth/resend"),
                        ("invitations", "/v1/invitations"),
                    ];

                    for (key, value) in defaults {
                        routes
                            .entry(key.to_string())
                            .or_insert_with(|| toml::Value::String(value.to_string()));
                    }
                }

                version = 2;
            }
            _ => {
                return Err(format!("No migration available for config version {version}").into());
            }
        }

        config["config_version"] = toml::Value::Integer(version as i64);
    }

    let migrated = toml::to_string_pretty(&config)?;

    let parent = config_path
        .parent()
        .ok_or("Config file has no parent directory")?;

    // Write migrated config to a temporary file.
    let mut temp_file = NamedTempFile::new_in(parent)?;

    temp_file.write_all(migrated.as_bytes())?;
    temp_file.as_file().sync_all()?;

    // Back up the original config.
    let backup_path = config_path.with_extension("toml.bak");

    if !backup_path.exists() {
        fs::copy(config_path, &backup_path)?;
    }

    // Atomically replace the original config.
    temp_file.persist(config_path)?;

    Ok(())
}
