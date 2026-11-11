use std::error::Error;
use std::path::Path;
use std::time::Duration;

use reqwest::{Client, Url};

use crate::settings::{ApiConfig, ApiRoutes};

/// Keep the client's resource ceiling predictable: short connect timeout,
/// generous but bounded request timeout, and a small bounded idle pool.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const POOL_MAX_IDLE_PER_HOST: usize = 8;

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

        let mut builder = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .pool_max_idle_per_host(POOL_MAX_IDLE_PER_HOST)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")));

        if let Some(path) = &config.cacert {
            let pem = std::fs::read_to_string(Path::new(path))
                .map_err(|error| format!("--cacert '{path}' could not be read: {error}"))?;
            let certificates = reqwest::Certificate::from_pem_bundle(pem.as_bytes())
                .map_err(|error| format!("--cacert '{path}' is not a valid PEM bundle: {error}"))?;

            for certificate in certificates {
                builder = builder.add_root_certificate(certificate);
            }
        }

        Ok(Self {
            client: builder.build()?,
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

/// Send an idempotent GET request with bounded backoff (3 attempts total).
/// Only retries connect/timeout errors and 5xx/429 responses — never for
/// writes, and never for 4xx.
pub async fn send_get_with_retry(
    build: impl Fn() -> reqwest::RequestBuilder,
) -> Result<reqwest::Response, reqwest::Error> {
    const BACKOFFS_MS: [u64; 2] = [250, 750];
    let mut attempt = 0usize;

    loop {
        match build().send().await {
            Ok(response) => {
                let retryable = response.status().is_server_error()
                    || response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS;

                if attempt < BACKOFFS_MS.len() && retryable {
                    attempt += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(BACKOFFS_MS[attempt - 1]))
                        .await;
                    continue;
                }

                return Ok(response);
            }
            Err(error) => {
                let retryable =
                    error.is_connect() || error.is_timeout() || error.is_request();

                if attempt < BACKOFFS_MS.len() && retryable {
                    attempt += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(BACKOFFS_MS[attempt - 1]))
                        .await;
                    continue;
                }

                return Err(error);
            }
        }
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
