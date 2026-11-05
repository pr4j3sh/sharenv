use super::client::ApiClient;
use crate::{
    models::{AddUserModel, AddUserRequest, CreateProjectModel, CreateProjectRequest},
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

    pub async fn add_user(
        &self,
        project_id: &str,
        email: String,
    ) -> Result<AddUserModel, Box<dyn std::error::Error>> {
        let mut request = self
            .client
            .put(self.resource(&self.routes.projects, project_id))
            .json(&AddUserRequest { email });

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<AddUserModel>().await?)
    }
}
