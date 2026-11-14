use std::error::Error;

use super::client::ApiClient;
use crate::{
    models::{LogoutModel, LoginModel, RegisterModel, ResendModel, SessionTokens},
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

        let request = self.client.post(self.url(&self.routes.register)).json(&payload);

        self.send_json(request).await
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

        let request = self.client.post(self.url(&self.routes.login)).json(&payload);

        self.send_json(request).await
    }

    /// Exchange a refresh token for a new token pair. The refresh token is
    /// single-use; a successful call rotates it.
    pub async fn refresh(
        &self,
        refresh_token: &str,
    ) -> Result<SessionTokens, Box<dyn Error>> {
        let request = self
            .client
            .post(self.url(&self.routes.refresh))
            .json(&crate::models::RefreshRequest {
                refresh_token: refresh_token.to_string(),
            });

        self.send_json(request).await
    }

    pub async fn verify(
        &self,
        email: &str,
        otp: &str,
    ) -> Result<SessionTokens, Box<dyn Error>> {
        let email = email.trim().to_lowercase();
        let request = self
            .client
            .post(self.url(&self.routes.verify))
            .json(&crate::models::VerifyRequest {
                email,
                otp: otp.to_string(),
            });

        self.send_json(request).await
    }

    pub async fn resend(&self, email: &str) -> Result<ResendModel, Box<dyn Error>> {
        let email = email.trim().to_lowercase();
        let request = self
            .client
            .post(self.url(&self.routes.resend))
            .json(&crate::models::ResendRequest { email });

        self.send_json(request).await
    }

    /// Revoke every token issued to the current user. Requires a live
    /// bearer token.
    pub async fn logout(&self) -> Result<LogoutModel, Box<dyn Error>> {
        let request = self.client.post(self.url(&self.routes.logout));

        self.send_json(self.bearer(request)).await
    }
}
