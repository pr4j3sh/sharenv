use super::client::ApiClient;
use crate::models::{
    CreateVariableModel, CreateVariableRequest, UpdateVariableModel, UpdateVariableRequest,
    VariableModel,
};

impl ApiClient {
    pub async fn get_variables(
        &self,
        environment_id: &str,
    ) -> Result<Vec<VariableModel>, Box<dyn std::error::Error>> {
        let url = self.resource(&self.routes.variables, environment_id);
        self.get_paginated(&url, &[]).await
    }

    pub async fn create_variable(
        &self,
        environment_id: &str,
        name: String,
        value: String,
    ) -> Result<CreateVariableModel, Box<dyn std::error::Error>> {
        let request = self
            .client
            .post(self.resource(&self.routes.variables, environment_id))
            .json(&CreateVariableRequest { name, value });

        self.send_json(self.bearer(request)).await
    }

    pub async fn update_variable(
        &self,
        variable_id: &str,
        value: String,
    ) -> Result<UpdateVariableModel, Box<dyn std::error::Error>> {
        let request = self
            .client
            .put(self.resource(&self.routes.variables, variable_id))
            .json(&UpdateVariableRequest { value });

        self.send_json(self.bearer(request)).await
    }
}
