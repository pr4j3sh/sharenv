use serde::Serialize;

use super::client::ApiClient;
use crate::{
    models::{CreateProjectModel, ProjectModel},
    utils::error::api_error,
};

#[derive(Serialize, Debug)]
struct CreateProjectRequest {
    name: String,
}

impl ApiClient {
    pub async fn create_project(
        &self,
        name: String,
    ) -> Result<CreateProjectModel, Box<dyn std::error::Error>> {
        let mut request = self
            .client
            .post(self.url("/v1/project/"))
            .json(&CreateProjectRequest { name });

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(api_error(response).await);
        }

        Ok(response.json::<CreateProjectModel>().await?)
    }

    pub async fn get_projects(&self) -> Result<Vec<ProjectModel>, reqwest::Error> {
        let mut request = self.client.get(self.url("/v1/project/"));

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        response
            .error_for_status()?
            .json::<Vec<ProjectModel>>()
            .await
    }
}
