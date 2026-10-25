use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub mongodb: MongoConfig,
    pub environment_files: EnvironmentFiles,
}

#[derive(Debug, Deserialize)]
pub struct MongoConfig {
    pub uri: String,
    pub database: String,
}

#[derive(Debug, Deserialize)]
pub struct EnvironmentFiles {
    pub files: Vec<String>,
}

impl AppConfig {
    pub fn load() -> Result<Self, ::config::ConfigError> {
        ::config::Config::builder()
            .add_source(::config::File::with_name("config").required(false))
            .add_source(::config::Environment::with_prefix("SHARENV").separator("__"))
            .build()?
            .try_deserialize()
    }
}
