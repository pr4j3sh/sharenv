use serde::Deserialize;

use super::version::CURRENT_CONFIG_VERSION;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_config_version")]
    pub config_version: u32,
    pub config_dir_name: String,
    pub config_file_path: String,
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
    pub env_file_prefix: String,
}

fn default_config_version() -> u32 {
    CURRENT_CONFIG_VERSION
}
