use super::client::ApiClient;
use crate::{
    models::{CreateEnvironmentModel, CreateEnvironmentRequest, EnvironmentModel},
    utils::error::api_error,
};

impl ApiClient {
    pub async fn get_environments(
        &self,
        project_id: &str,
        environment_name: Option<&str>,
    ) -> Result<Vec<EnvironmentModel>, Box<dyn std::error::Error>> {
        let mut request = self
            .client
            .get(self.resource(&self.routes.environments, project_id));

        if let Some(environment_name) = environment_name {
            request = request.query(&[(
                self.routes.environment_name_param.as_str(),
                environment_name,
            )]);
        }

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<Vec<EnvironmentModel>>().await?)
    }

    pub async fn create_environment(
        &self,
        project_id: &str,
        name: String,
    ) -> Result<CreateEnvironmentModel, Box<dyn std::error::Error>> {
        let mut request = self
            .client
            .post(self.resource(&self.routes.environments, project_id))
            .json(&CreateEnvironmentRequest { name });

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<CreateEnvironmentModel>().await?)
    }
}
