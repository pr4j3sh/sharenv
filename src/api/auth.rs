use std::error::Error;

use serde::Serialize;

use super::client::ApiClient;
use crate::{models::AuthModel, utils::error::api_error};

#[derive(Serialize, Debug)]
struct AuthRequest {
    email: String,
    password: String,
}

impl ApiClient {
    pub async fn register(
        &self,
        email: String,
        password: String,
    ) -> Result<AuthModel, Box<dyn Error>> {
        let payload = AuthRequest { email, password };

        let response = self
            .client
            .post(self.url("/v1/auth/register"))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(api_error(response).await);
        }

        Ok(response.json::<AuthModel>().await?)
    }
    pub async fn login(
        &self,
        email: String,
        password: String,
    ) -> Result<AuthModel, Box<dyn Error>> {
        let payload = AuthRequest { email, password };

        let response = self
            .client
            .post(self.url("/v1/auth/login"))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(api_error(response).await);
        }

        Ok(response.json::<AuthModel>().await?)
    }
}
