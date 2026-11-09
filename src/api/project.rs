use super::client::ApiClient;
use crate::{
    models::{
        AddUserModel, AddUserRequest, CreateProjectModel, CreateProjectRequest, DeleteProjectModel,
        ProjectModel,
    },
    utils::error::api_error,
};

impl ApiClient {
    pub async fn get_projects(&self) -> Result<Vec<ProjectModel>, Box<dyn std::error::Error>> {
        let mut request = self.client.get(self.url(&self.routes.projects));

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<Vec<ProjectModel>>().await?)
    }

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

    pub async fn delete_project_by_name(
        &self,
        project_name: &str,
    ) -> Result<DeleteProjectModel, Box<dyn std::error::Error>> {
        let base = format!("{}/name", self.routes.projects.trim_end_matches('/'));
        let mut request = self.client.delete(self.resource(&base, project_name));

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<DeleteProjectModel>().await?)
    }

    pub async fn delete_project(
        &self,
        project_id: &str,
    ) -> Result<DeleteProjectModel, Box<dyn std::error::Error>> {
        let mut request = self
            .client
            .delete(self.resource(&self.routes.projects, project_id));

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<DeleteProjectModel>().await?)
    }
}
