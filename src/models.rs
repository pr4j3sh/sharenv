use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

fn deserialize_env_value<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;

    Ok(match value {
        Value::Null => None,
        Value::String(value) => Some(value),
        other => Some(other.to_string()),
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionTokens {
    pub token: String,
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginModel {
    pub token: String,
    pub refresh_token: String,
    pub email_verified: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterModel {
    pub success: bool,
    #[serde(default = "default_verification_required")]
    pub verification_required: bool,
}

fn default_verification_required() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResendModel {
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogoutModel {
    pub success: bool,
}

#[derive(Serialize, Debug)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Debug)]
pub struct CreateProjectRequest {
    pub name: String,
}

#[derive(Serialize, Debug)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize, Debug)]
pub struct VerifyRequest {
    pub email: String,
    pub otp: String,
}

#[derive(Serialize, Debug)]
pub struct ResendRequest {
    pub email: String,
}

#[derive(Serialize, Debug)]
pub struct DeleteProjectRequest {
    pub password: String,
}

#[derive(Serialize, Debug)]
pub struct CreateInvitationRequest {
    pub email: String,
    pub role: String,
    pub environments: Vec<String>,
    pub password: String,
}

#[derive(Serialize, Debug)]
pub struct UpdateMemberAccessRequest {
    pub environments: Vec<String>,
    pub role: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct MemberModel {
    pub user_id: String,
    pub email: String,
    pub role: String,
    #[serde(default)]
    pub environments: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateMemberAccessModel {
    #[allow(dead_code)]
    pub project_id: String,
    #[allow(dead_code)]
    pub user_id: String,
    pub role: String,
    #[serde(default)]
    pub environments: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct CancelInvitationRequest {
    pub password: String,
}

#[derive(Serialize, Debug)]
pub struct CreateEnvironmentRequest {
    pub name: String,
}

#[derive(Serialize, Debug)]
pub struct CreateVariableRequest {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Debug)]
pub struct UpdateVariableRequest {
    pub value: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateProjectModel {
    pub project_id: String,
}

#[derive(Deserialize, Debug)]
pub struct DeleteProjectModel {
    pub project_id: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateInvitationModel {
    pub invitation_id: String,
    #[allow(dead_code)]
    pub project_id: String,
    pub email: String,
    pub role: String,
    pub environments: Vec<String>,
    #[allow(dead_code)]
    pub expires_at: DateTime<Utc>,
    /// The raw invitation token. Returned only at creation time and stored
    /// hashed server-side; this is the only time it is exposed.
    pub token: String,
    pub email_sent: bool,
}

#[derive(Deserialize, Debug)]
pub struct InvitationModel {
    pub _id: Option<String>,
    #[allow(dead_code)]
    pub project_id: String,
    pub email: String,
    pub role: String,
    #[serde(default)]
    pub environments: Vec<String>,
    #[allow(dead_code)]
    pub invited_by: String,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Debug)]
pub struct AcceptInvitationModel {
    pub project_id: String,
    pub role: String,
    #[serde(default)]
    pub environments: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct CancelInvitationModel {
    #[allow(dead_code)]
    pub invitation_id: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateEnvironmentModel {
    pub environment_id: String,
}

#[derive(Deserialize, Debug)]
pub struct DeleteEnvironmentModel {
    pub environment_id: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateVariableModel {
    #[allow(dead_code)]
    pub variable_id: String,
}

#[derive(Deserialize, Debug)]
pub struct UpdateVariableModel {
    #[allow(dead_code)]
    pub variable_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectModel {
    pub _id: String,
    pub name: String,
    pub user_id: String,
    #[serde(default = "default_role")]
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_role() -> String {
    "owner".to_string()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnvironmentModel {
    pub _id: String,
    pub name: String,
    pub project_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VariableModel {
    pub _id: String,
    pub name: String,
    #[serde(deserialize_with = "deserialize_env_value")]
    pub value: Option<String>,
    pub environment_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub undecryptable: bool,
}
