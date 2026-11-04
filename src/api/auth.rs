use std::error::Error;

use super::client::ApiClient;
use crate::{
    models::{AuthModel, AuthRequest},
    utils::error::api_error,
};

impl ApiClient {
    pub async fn register(
        &self,
        email: String,
        password: String,
    ) -> Result<AuthModel, Box<dyn Error>> {
        let payload = AuthRequest { email, password };

        let response = self
            .client
            .post(self.url(&self.routes.register))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
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
            .post(self.url(&self.routes.login))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<AuthModel>().await?)
    }
}
