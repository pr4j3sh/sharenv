use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub api_config: ApiConfig,
    pub auth: AuthConfig,
    pub environment_files: EnvironmentFilesConfig,
}

#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
    pub routes: ApiRoutes,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiRoutes {
    pub register: String,
    pub login: String,
    pub projects: String,
    pub environments: String,
    pub variables: String,
    pub environment_name_param: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub token_service: String,
    pub token_account: String,
}

#[derive(Debug, Deserialize)]
pub struct EnvironmentFilesConfig {
    pub environments: Vec<String>,
    pub config_dir_name: String,
    pub config_file_path: String,
    pub env_file_prefix: String,
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
