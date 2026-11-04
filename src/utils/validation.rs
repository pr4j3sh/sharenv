pub fn validate_project_name(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    validate_name("project", name)
}

pub fn validate_environment_name(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    validate_name("environment", name)
}

fn validate_name(kind: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let valid = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');

    if valid {
        Ok(())
    } else {
        Err(format!(
            "invalid {kind} name '{name}': use only lowercase letters, numbers, hyphens, and underscores"
        )
        .into())
    }
}
