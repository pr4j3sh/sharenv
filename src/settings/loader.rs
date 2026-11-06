use super::migrations::migrate_config;
use super::models::AppConfig;
use super::paths::ensure_config_exists;

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = ensure_config_exists()?;

        migrate_config(&config_path)?;

        let config = ::config::Config::builder()
            .add_source(::config::File::from(config_path).required(true))
            .add_source(::config::Environment::with_prefix("SHARENV").separator("__"))
            .build()?
            .try_deserialize()?;

        Ok(config)
    }
}
