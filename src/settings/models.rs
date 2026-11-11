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
    /// Runtime-only `--profile` switch, like `insecure` it is deliberately
    /// skipped during deserialization so it cannot be persisted in config or
    /// supplied through an environment override.
    #[serde(skip)]
    pub profile: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
    pub routes: ApiRoutes,
    /// Runtime-only escape hatch set by the global `--insecure` switch.
    /// Deliberately skipped during deserialization so it cannot be persisted
    /// in config or supplied through an environment override.
    #[serde(skip)]
    pub insecure: bool,
    /// Runtime-only `--cacert <path>` for self-hosted deployments with a
    /// private CA. Never persisted in config or environment variables.
    #[serde(skip)]
    pub cacert: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiRoutes {
    pub register: String,
    pub login: String,
    pub refresh: String,
    pub logout: String,
    pub verify: String,
    pub resend: String,
    pub invitations: String,
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
