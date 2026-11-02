use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthModel {
    pub token: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateProjectModel {
    pub project_id: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateEnvironmentModel {
    pub environment_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectModel {
    pub _id: String,
    pub name: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub value: String,
    pub environment_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
