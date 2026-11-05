use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

fn deserialize_env_value<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;

    Ok(match value {
        Value::String(value) => value,
        other => other.to_string(),
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthModel {
    pub token: String,
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
pub struct AddUserRequest {
    pub email: String,
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

#[derive(Deserialize, Debug)]
pub struct CreateProjectModel {
    pub project_id: String,
}

#[derive(Deserialize, Debug)]
pub struct AddUserModel {
    pub project_id: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateEnvironmentModel {
    pub environment_id: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateVariableModel {
    #[allow(dead_code)]
    pub variable_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectModel {
    pub _id: String,
    pub name: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub users: Vec<String>,
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
    pub value: String,
    pub environment_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
