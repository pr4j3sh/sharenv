use super::client::ApiClient;
use crate::{
    models::{
        CreateProjectModel, CreateProjectRequest, DeleteProjectModel, DeleteProjectRequest,
        ProjectModel,
    },
    utils::error::api_error,
};

impl ApiClient {
    pub async fn get_projects(&self) -> Result<Vec<ProjectModel>, Box<dyn std::error::Error>> {
        let mut projects = Vec::new();
        let mut offset = 0u32;

        loop {
            let limit = 100u32;
            let response = crate::api::client::send_get_with_retry(|| {
                let mut request = self
                    .client
                    .get(self.url(&self.routes.projects))
                    .query(&[("limit", limit), ("offset", offset)]);

                if let Some(token) = &self.token {
                    request = request.bearer_auth(token);
                }

                request
            })
            .await?;

            if !response.status().is_success() {
                return Err(Box::new(api_error(response).await));
            }

            let page: Vec<ProjectModel> = response.json().await?;
            let count = page.len();
            projects.extend(page);

            if (count as u32) < limit {
                break;
            }

            offset += limit;
        }

        Ok(projects)
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

    pub async fn delete_project_by_name(
        &self,
        project_name: &str,
        password: &str,
    ) -> Result<DeleteProjectModel, Box<dyn std::error::Error>> {
        let base = format!("{}/name", self.routes.projects.trim_end_matches('/'));
        let mut request = self
            .client
            .delete(self.resource(&base, project_name))
            .json(&DeleteProjectRequest {
                password: password.to_string(),
            });

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
        password: &str,
    ) -> Result<DeleteProjectModel, Box<dyn std::error::Error>> {
        let mut request = self
            .client
            .delete(self.resource(&self.routes.projects, project_id))
            .json(&DeleteProjectRequest {
                password: password.to_string(),
            });

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
