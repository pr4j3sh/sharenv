use super::client::ApiClient;
use crate::models::{
    CreateProjectModel, CreateProjectRequest, DeleteProjectModel, DeleteProjectRequest,
    ProjectModel,
};

impl ApiClient {
    pub async fn get_projects(&self) -> Result<Vec<ProjectModel>, Box<dyn std::error::Error>> {
        let url = self.url(&self.routes.projects);
        self.get_paginated(&url, &[]).await
    }

    pub async fn create_project(
        &self,
        name: String,
    ) -> Result<CreateProjectModel, Box<dyn std::error::Error>> {
        let request = self
            .client
            .post(self.url(&self.routes.projects))
            .json(&CreateProjectRequest { name });

        self.send_json(self.bearer(request)).await
    }

    pub async fn delete_project_by_name(
        &self,
        project_name: &str,
        password: &str,
    ) -> Result<DeleteProjectModel, Box<dyn std::error::Error>> {
        let base = format!("{}/name", self.routes.projects.trim_end_matches('/'));
        let request = self
            .client
            .delete(self.resource(&base, project_name))
            .json(&DeleteProjectRequest {
                password: password.to_string(),
            });

        self.send_json(self.bearer(request)).await
    }

    pub async fn delete_project(
        &self,
        project_id: &str,
        password: &str,
    ) -> Result<DeleteProjectModel, Box<dyn std::error::Error>> {
        let request = self
            .client
            .delete(self.resource(&self.routes.projects, project_id))
            .json(&DeleteProjectRequest {
                password: password.to_string(),
            });

        self.send_json(self.bearer(request)).await
    }
}
