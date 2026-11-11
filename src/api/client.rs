use std::error::Error;

use reqwest::{Client, Url};

use crate::settings::{ApiConfig, ApiRoutes};

pub struct ApiClient {
    pub client: Client,
    pub base_url: String,
    pub routes: ApiRoutes,
    pub token: Option<String>,
}

impl ApiClient {
    pub fn new(config: &ApiConfig) -> Result<Self, Box<dyn Error>> {
        let base_url = config.base_url.trim_end_matches('/').to_string();
        validate_base_url(&base_url, config.insecure)?;

        Ok(Self {
            client: Client::new(),
            base_url,
            routes: config.routes.clone(),
            token: None,
        })
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

/// Refuse to send credentials over plaintext HTTP unless the host is local or
/// the user explicitly passed `--insecure`.
pub fn validate_base_url(raw: &str, insecure: bool) -> Result<(), Box<dyn Error>> {
    let url =
        Url::parse(raw).map_err(|error| format!("invalid api_config.base_url '{raw}': {error}"))?;

    let host = url.host_str().unwrap_or("<unknown>");

    match url.scheme() {
        "https" => Ok(()),
        "http" if is_loopback_host(host) => Ok(()),
        "http" if insecure => {
            eprintln!(
                "warning: --insecure set; credentials and secrets will be sent unencrypted to http://{host}"
            );
            Ok(())
        }
        "http" => Err(format!(
            "refusing to send credentials over plaintext HTTP to '{host}'; \
             use https:// or pass --insecure to override (not recommended)"
        )
        .into()),
        scheme => {
            Err(format!("unsupported api_config.base_url scheme '{scheme}'; use https://").into())
        }
    }
}

fn is_loopback_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }

    let host = host.trim_start_matches('[').trim_end_matches(']');

    host.parse::<std::net::IpAddr>()
        .is_ok_and(|address| address.is_loopback())
}
