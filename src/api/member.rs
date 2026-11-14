use super::client::ApiClient;
use crate::models::{MemberModel, UpdateMemberAccessModel, UpdateMemberAccessRequest};

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
        let url = self.project_members_url(project_id);

        Ok(self
            .get_with_retry(|| self.bearer(self.client.get(&url)))
            .await?
            .json::<Vec<MemberModel>>()
            .await?)
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
        let request = self
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

        self.send_json(self.bearer(request)).await
    }
}
