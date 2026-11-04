use std::collections::HashMap;

use crate::{models::VariableModel, utils::env_parser::EnvVariable};

pub struct ChangePlan {
    pub additions: Vec<EnvVariable>,
    pub updates: Vec<ValueChange>,
}

pub struct ValueChange {
    pub name: String,
    pub current: String,
    pub remote: String,
}

pub fn plan_changes(remote: &[VariableModel], local: &[EnvVariable]) -> ChangePlan {
    let local_values: HashMap<&str, &str> = local
        .iter()
        .map(|variable| (variable.name.as_str(), variable.value.as_str()))
        .collect();

    let mut additions = Vec::new();
    let mut updates = Vec::new();

    for variable in remote {
        match local_values.get(variable.name.as_str()) {
            None => additions.push(EnvVariable {
                name: variable.name.clone(),
                value: variable.value.clone(),
            }),
            Some(current) if *current != variable.value => updates.push(ValueChange {
                name: variable.name.clone(),
                current: (*current).to_string(),
                remote: variable.value.clone(),
            }),
            Some(_) => {}
        }
    }

    ChangePlan { additions, updates }
}
