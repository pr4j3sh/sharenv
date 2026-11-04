use super::client::ApiClient;
use crate::{
    models::{CreateProjectModel, CreateProjectRequest},
    utils::error::api_error,
};

impl ApiClient {
    pub async fn create_project(
        &self,
        name: String,
    ) -> Result<CreateProjectModel, Box<dyn std::error::Error>> {
        let mut request = self
            .client
            .post(self.url(&self.routes.projects))
            .json(&CreateProjectRequest { name });

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<CreateProjectModel>().await?)
    }
}
