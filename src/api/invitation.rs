use super::client::ApiClient;
use crate::{
    models::{
        AcceptInvitationModel, CancelInvitationModel, CancelInvitationRequest,
        CreateInvitationModel, CreateInvitationRequest, InvitationModel,
    },
    utils::error::api_error,
};

impl ApiClient {
    fn project_invitations_url(&self, project_id: &str) -> String {
        self.url(&format!(
            "{}/{}/invitations",
            self.routes.projects.trim_end_matches('/'),
            project_id
        ))
    }

    pub async fn create_invitation(
        &self,
        project_id: &str,
        email: String,
        role: &str,
        environments: Vec<String>,
        password: &str,
    ) -> Result<CreateInvitationModel, Box<dyn std::error::Error>> {
        let mut request = self
            .client
            .post(self.project_invitations_url(project_id))
            .json(&CreateInvitationRequest {
                email,
                role: role.to_string(),
                environments,
                password: password.to_string(),
            });

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<CreateInvitationModel>().await?)
    }

    pub async fn list_invitations(
        &self,
        project_id: &str,
    ) -> Result<Vec<InvitationModel>, Box<dyn std::error::Error>> {
        let mut request = self.client.get(self.project_invitations_url(project_id));

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<Vec<InvitationModel>>().await?)
    }

    pub async fn cancel_invitation(
        &self,
        project_id: &str,
        invitation_id: &str,
        password: &str,
    ) -> Result<CancelInvitationModel, Box<dyn std::error::Error>> {
        let mut request = self
            .client
            .delete(format!(
                "{}/{}",
                self.project_invitations_url(project_id),
                invitation_id
            ))
            .json(&CancelInvitationRequest {
                password: password.to_string(),
            });

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<CancelInvitationModel>().await?)
    }

    pub async fn accept_invitation(
        &self,
        token: &str,
    ) -> Result<AcceptInvitationModel, Box<dyn std::error::Error>> {
        let mut request = self
            .client
            .post(format!("{}/{}/accept", self.url(&self.routes.invitations), token));

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<AcceptInvitationModel>().await?)
    }
}
