use std::collections::HashMap;

use crate::{models::VariableModel, utils::env_parser::EnvVariable, utils::output::sanitize};

/// Split out remote variables whose values the server could not decrypt and
/// report each one. They are skipped everywhere: they cannot be diffed and
/// must never be written into a local .env file.
pub fn filter_decryptable(
    remote: Vec<VariableModel>,
) -> (Vec<VariableModel>, Vec<VariableModel>) {
    let mut usable = Vec::with_capacity(remote.len());
    let mut skipped = Vec::new();

    for variable in remote {
        let undecryptable = variable.undecryptable || variable.value.is_none();

        if undecryptable {
            eprintln!(
                "warning: skipping '{}': the server could not decrypt this value",
                sanitize(&variable.name)
            );
            skipped.push(variable);
        } else {
            usable.push(variable);
        }
    }

    (usable, skipped)
}

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
        // Undecryptable remote values cannot be diffed or synced; callers
        // report them separately.
        let Some(remote_value) = variable.value.as_deref() else {
            continue;
        };

        match local_values.get(variable.name.as_str()) {
            None => additions.push(EnvVariable {
                name: variable.name.clone(),
                value: remote_value.to_string(),
            }),
            Some(current) if *current != remote_value => updates.push(ValueChange {
                name: variable.name.clone(),
                current: (*current).to_string(),
                remote: remote_value.to_string(),
            }),
            Some(_) => {}
        }
    }

    ChangePlan { additions, updates }
}
