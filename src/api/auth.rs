use std::error::Error;

use super::client::ApiClient;
use crate::{
    models::{LogoutModel, LoginModel, RegisterModel, ResendModel, SessionTokens},
    utils::error::api_error,
};

impl ApiClient {
    pub async fn register(
        &self,
        email: &str,
        password: &str,
    ) -> Result<RegisterModel, Box<dyn Error>> {
        let payload = crate::models::AuthRequest {
            email: email.to_string(),
            password: password.to_string(),
        };

        let response = self
            .client
            .post(self.url(&self.routes.register))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<RegisterModel>().await?)
    }

    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<LoginModel, Box<dyn Error>> {
        let payload = crate::models::AuthRequest {
            email: email.to_string(),
            password: password.to_string(),
        };

        let response = self
            .client
            .post(self.url(&self.routes.login))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<LoginModel>().await?)
    }

    /// Exchange a refresh token for a new token pair. The refresh token is
    /// single-use; a successful call rotates it.
    pub async fn refresh(
        &self,
        refresh_token: &str,
    ) -> Result<SessionTokens, Box<dyn Error>> {
        let response = self
            .client
            .post(self.url(&self.routes.refresh))
            .json(&crate::models::RefreshRequest {
                refresh_token: refresh_token.to_string(),
            })
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<SessionTokens>().await?)
    }

    pub async fn verify(
        &self,
        email: &str,
        otp: &str,
    ) -> Result<SessionTokens, Box<dyn Error>> {
        let email = email.trim().to_lowercase();
        let response = self
            .client
            .post(self.url(&self.routes.verify))
            .json(&crate::models::VerifyRequest {
                email: email.clone(),
                otp: otp.to_string(),
            })
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<SessionTokens>().await?)
    }

    pub async fn resend(&self, email: &str) -> Result<ResendModel, Box<dyn Error>> {
        let email = email.trim().to_lowercase();
        let response = self
            .client
            .post(self.url(&self.routes.resend))
            .json(&crate::models::ResendRequest { email })
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<ResendModel>().await?)
    }

    /// Revoke every token issued to the current user. Requires a live
    /// bearer token.
    pub async fn logout(&self) -> Result<LogoutModel, Box<dyn Error>> {
        let mut request = self.client.post(self.url(&self.routes.logout));

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<LogoutModel>().await?)
    }
}
