use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EnvironmentVariable {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Environment {
    pub name: String,
    pub variables: Vec<EnvironmentVariable>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub name: String,
    pub environments: Vec<Environment>,
}
