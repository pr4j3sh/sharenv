use super::client::ApiClient;
use crate::models::{
    CreateEnvironmentModel, CreateEnvironmentRequest, DeleteEnvironmentModel, EnvironmentModel,
};

impl ApiClient {
    pub async fn get_environments(
        &self,
        project_id: &str,
        environment_name: Option<&str>,
    ) -> Result<Vec<EnvironmentModel>, Box<dyn std::error::Error>> {
        let url = self.resource(&self.routes.environments, project_id);
        let extra_query = environment_name
            .map(|name| {
                vec![(self.routes.environment_name_param.clone(), name.to_string())]
            })
            .unwrap_or_default();

        self.get_paginated(&url, &extra_query).await
    }

    pub async fn create_environment(
        &self,
        project_id: &str,
        name: String,
    ) -> Result<CreateEnvironmentModel, Box<dyn std::error::Error>> {
        let request = self
            .client
            .post(self.resource(&self.routes.environments, project_id))
            .json(&CreateEnvironmentRequest { name });

        self.send_json(self.bearer(request)).await
    }

    pub async fn delete_environment(
        &self,
        environment_id: &str,
    ) -> Result<DeleteEnvironmentModel, Box<dyn std::error::Error>> {
        let request = self
            .client
            .delete(self.resource(&self.routes.environments, environment_id));

        self.send_json(self.bearer(request)).await
    }
}
