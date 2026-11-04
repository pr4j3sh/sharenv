use reqwest::Client;

use crate::settings::{ApiConfig, ApiRoutes};

pub struct ApiClient {
    pub client: Client,
    pub base_url: String,
    pub routes: ApiRoutes,
    pub token: Option<String>,
}

impl ApiClient {
    pub fn new(config: &ApiConfig) -> Self {
        Self {
            client: Client::new(),
            base_url: config.base_url.trim_end_matches('/').to_string(),
            routes: config.routes.clone(),
            token: None,
        }
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }

    pub fn resource(&self, base: &str, id: &str) -> String {
        self.url(&format!("{}/{}", base.trim_end_matches('/'), id))
    }
}
