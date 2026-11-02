use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub api_config: ApiConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    pub environment_files: DefaultConfig,
}

#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub token_service: String,
    pub token_account: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            token_service: "sharenv".to_string(),
            token_account: "default".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DefaultConfig {
    pub environments: Vec<String>,
    pub config_dir_name: String,
    pub config_file_path: String,
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
