pub fn validate_project_name(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    validate_name("project", name)
}

/// Match the server's allowlist for variable names exactly:
/// `^[A-Za-z_][A-Za-z0-9_]*$`, max 256 bytes. This blocks names with
/// newlines, `=`, `#`, `/`, spaces, quotes and control characters.
pub fn validate_variable_name(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let first_byte = name.as_bytes().first().copied();
    let first_valid = first_byte.is_some_and(|b| b.is_ascii_alphabetic() || b == b'_');
    let rest_valid = name.chars().skip(1).all(|c| {
        c.is_ascii_alphanumeric() || c == '_'
    });

    if !name.is_empty() && name.len() <= 256 && first_valid && rest_valid {
        return Ok(());
    }

    Err(format!(
        "invalid variable name '{}': must match ^[A-Za-z_][A-Za-z0-9_]*$",
        crate::utils::output::sanitize(name)
    )
    .into())
}

pub fn validate_email(email: &str) -> Result<(), Box<dyn std::error::Error>> {
    let valid = !email.is_empty()
        && email.len() <= 254
        && email.chars().filter(|c| *c == '@').count() == 1
        && !email.starts_with('@')
        && !email.ends_with('@')
        && email.chars().all(|c| !c.is_whitespace() && !c.is_control());

    if valid {
        Ok(())
    } else {
        Err(format!("invalid email '{}'", crate::utils::output::sanitize(email)).into())
    }
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
