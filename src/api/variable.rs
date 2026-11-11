use super::client::ApiClient;
use crate::{
    models::{
        CreateVariableModel, CreateVariableRequest, UpdateVariableModel, UpdateVariableRequest,
        VariableModel,
    },
    utils::error::api_error,
};

impl ApiClient {
    pub async fn get_variables(
        &self,
        environment_id: &str,
    ) -> Result<Vec<VariableModel>, Box<dyn std::error::Error>> {
        let mut variables = Vec::new();
        let mut offset = 0u32;

        loop {
            let limit = 100u32;
            let response = crate::api::client::send_get_with_retry(|| {
                let mut request = self
                    .client
                    .get(self.resource(&self.routes.variables, environment_id))
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

            let page: Vec<VariableModel> = response.json().await?;
            let count = page.len();
            variables.extend(page);

            if (count as u32) < limit {
                break;
            }

            offset += limit;
        }

        Ok(variables)
    }

    pub async fn create_variable(
        &self,
        environment_id: &str,
        name: String,
        value: String,
    ) -> Result<CreateVariableModel, Box<dyn std::error::Error>> {
        let mut request = self
            .client
            .post(self.resource(&self.routes.variables, environment_id))
            .json(&CreateVariableRequest { name, value });

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<CreateVariableModel>().await?)
    }

    pub async fn update_variable(
        &self,
        variable_id: &str,
        value: String,
    ) -> Result<UpdateVariableModel, Box<dyn std::error::Error>> {
        let mut request = self
            .client
            .put(self.resource(&self.routes.variables, variable_id))
            .json(&UpdateVariableRequest { value });

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<UpdateVariableModel>().await?)
    }
}
