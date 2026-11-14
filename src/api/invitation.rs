use super::client::ApiClient;
use crate::models::{
    AcceptInvitationModel, CancelInvitationModel, CancelInvitationRequest,
    CreateInvitationModel, CreateInvitationRequest, InvitationModel,
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
        let request = self
            .client
            .post(self.project_invitations_url(project_id))
            .json(&CreateInvitationRequest {
                email,
                role: role.to_string(),
                environments,
                password: password.to_string(),
            });

        self.send_json(self.bearer(request)).await
    }

    pub async fn list_invitations(
        &self,
        project_id: &str,
    ) -> Result<Vec<InvitationModel>, Box<dyn std::error::Error>> {
        let url = self.project_invitations_url(project_id);

        Ok(self
            .get_with_retry(|| self.bearer(self.client.get(&url)))
            .await?
            .json::<Vec<InvitationModel>>()
            .await?)
    }

    pub async fn cancel_invitation(
        &self,
        project_id: &str,
        invitation_id: &str,
        password: &str,
    ) -> Result<CancelInvitationModel, Box<dyn std::error::Error>> {
        let request = self
            .client
            .delete(format!(
                "{}/{}",
                self.project_invitations_url(project_id),
                invitation_id
            ))
            .json(&CancelInvitationRequest {
                password: password.to_string(),
            });

        self.send_json(self.bearer(request)).await
    }

    pub async fn accept_invitation(
        &self,
        token: &str,
    ) -> Result<AcceptInvitationModel, Box<dyn std::error::Error>> {
        let request = self
            .client
            .post(format!("{}/{}/accept", self.url(&self.routes.invitations), token));

        self.send_json(self.bearer(request)).await
    }
}
