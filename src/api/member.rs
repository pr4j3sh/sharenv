use super::client::ApiClient;
use crate::{
    models::{MemberModel, UpdateMemberAccessModel, UpdateMemberAccessRequest},
    utils::error::api_error,
};

impl ApiClient {
    fn project_members_url(&self, project_id: &str) -> String {
        self.url(&format!(
            "{}/{}/members",
            self.routes.projects.trim_end_matches('/'),
            project_id
        ))
    }

    /// Owner-only: lists the project owner and all members with roles and
    /// environment grants.
    pub async fn list_members(
        &self,
        project_id: &str,
    ) -> Result<Vec<MemberModel>, Box<dyn std::error::Error>> {
        let response = crate::api::client::send_get_with_retry(|| {
            let mut request = self.client.get(self.project_members_url(project_id));

            if let Some(token) = &self.token {
                request = request.bearer_auth(token);
            }

            request
        })
        .await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<Vec<MemberModel>>().await?)
    }

    /// Owner-only: fully replace a member's environment grants and optionally
    /// change their role. Re-authenticates with the owner's password.
    pub async fn update_member_access(
        &self,
        project_id: &str,
        member_id: &str,
        environments: Vec<String>,
        role: &str,
        password: &str,
    ) -> Result<UpdateMemberAccessModel, Box<dyn std::error::Error>> {
        let mut request = self
            .client
            .patch(format!(
                "{}/{}",
                self.project_members_url(project_id),
                member_id
            ))
            .json(&UpdateMemberAccessRequest {
                environments,
                role: role.to_string(),
                password: password.to_string(),
            });

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Box::new(api_error(response).await));
        }

        Ok(response.json::<UpdateMemberAccessModel>().await?)
    }
}
