use serde::Serialize;

use super::client::ApiClient;
use crate::{models::CreateEnvironmentModel, utils::error::api_error};

#[derive(Serialize, Debug)]
struct CreateEnvironmentRequest {
    name: String,
}

impl ApiClient {
    pub async fn create_environment(
        &self,
        project_id: &str,
        name: String,
    ) -> Result<CreateEnvironmentModel, Box<dyn std::error::Error>> {
        let mut request = self
            .client
            .post(self.url(&format!("/v1/environment/{project_id}")))
            .json(&CreateEnvironmentRequest { name });

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(api_error(response).await);
        }

        Ok(response.json::<CreateEnvironmentModel>().await?)
    }
}
