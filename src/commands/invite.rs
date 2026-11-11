use std::error::Error;

use crate::{
    settings::AppConfig,
    utils::{
        local_config,
        output::sanitize,
        prompt::prompt_line,
        session,
        validation::validate_email,
    },
};

pub async fn run(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    let (email, role_flag) = parse_args(args)?;

    validate_email(&email)?;

    let role = match role_flag {
        Some(role) => {
            let role = role.to_ascii_lowercase();

            if role != "writer" && role != "reader" {
                return Err("--role must be 'writer' or 'reader'".into());
            }

            role
        }
        None => {
            let input = prompt_line("role (writer/reader) [reader]: ")?;

            match input.to_ascii_lowercase().as_str() {
                "" | "reader" => "reader".to_string(),
                "writer" => "writer".to_string(),
                _ => return Err("role must be 'writer' or 'reader'".into()),
            }
        }
    };

    let local =
        local_config::load(config).map_err(|_| format!("no {} found", config.config_file_path))?;

    if local.project.id.is_empty() {
        return Err(format!(
            "project id is missing from {}; run '{} init' first",
            config.config_file_path,
            env!("CARGO_PKG_NAME")
        )
        .into());
    }

    let client = session::authenticated_client(config).await?;

    let environments = client.get_environments(&local.project.id, None).await?;

    if environments.is_empty() {
        return Err("this project has no environments to grant".into());
    }

    println!("Environments in this project:");

    for (index, environment) in environments.iter().enumerate() {
        println!("  {}. {}", index + 1, sanitize(&environment.name));
    }

    let selection = prompt_line("grant access to which environments? (names, 'all') [all]: ")?;
    let granted = pick_environments(&selection, &environments)?;

    let password = rpassword::prompt_password("your password (required to share): ")?;

    if password.is_empty() {
        return Err("password cannot be empty".into());
    }

    let invitation = client
        .create_invitation(
            &local.project.id,
            email.to_string(),
            &role,
            granted,
            &password,
        )
        .await?;

    let granted_names = environments_names_for(&invitation.environments, &environments);

    println!(
        "Invitation created for '{}' (role: {}, environments: {}).",
        sanitize(&invitation.email),
        invitation.role,
        granted_names.join(", ")
    );
    println!("Invitation id: {}", invitation.invitation_id);

    if invitation.email_sent {
        println!("An invitation email was sent to '{}'.", sanitize(&invitation.email));
    } else {
        println!("No mail provider is configured on the server, so no email was sent.");
        println!("Share this invitation token with them out-of-band (shown only once):");
        println!();
        println!("    {}", invitation.token);
        println!();
        println!(
            "They can accept it with: {} invites accept <token>",
            env!("CARGO_PKG_NAME")
        );
    }

    Ok(())
}

fn parse_args(args: &[String]) -> Result<(String, Option<String>), Box<dyn Error>> {
    let mut email: Option<String> = None;
    let mut role: Option<String> = None;
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--role" => {
                let value = iter
                    .next()
                    .ok_or("missing value for --role")?;

                if value.starts_with('-') {
                    return Err("missing value for --role".into());
                }

                role = Some(value.clone());
            }
            value if value.starts_with("--role=") => {
                role = Some(value["--role=".len()..].to_string());
            }
            value if value.starts_with('-') => {
                return Err(format!("unexpected argument '{value}'").into());
            }
            value => {
                if email.is_some() {
                    return Err(format!("unexpected argument '{value}'").into());
                }

                email = Some(value.to_string());
            }
        }
    }

    let email = email.ok_or("missing email; usage: invite <email> [--role <writer|reader>]")?;

    Ok((email, role))
}

fn pick_environments(
    selection: &str,
    environments: &[crate::models::EnvironmentModel],
) -> Result<Vec<String>, Box<dyn Error>> {
    let trimmed = selection.trim();

    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("all") {
        return Ok(environments
            .iter()
            .map(|environment| environment._id.clone())
            .collect());
    }

    let mut ids = Vec::new();

    for name in trimmed.split(',').map(str::trim).filter(|name| !name.is_empty()) {
        let environment = environments
            .iter()
            .find(|environment| environment.name == name)
            .ok_or_else(|| {
                let valid = environments
                    .iter()
                    .map(|environment| environment.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("unknown environment '{name}'; valid: {valid}")
            })?;

        if !ids.contains(&environment._id) {
            ids.push(environment._id.clone());
        }
    }

    Ok(ids)
}

fn environments_names_for(
    ids: &[String],
    environments: &[crate::models::EnvironmentModel],
) -> Vec<String> {
    ids.iter()
        .map(|id| {
            environments
                .iter()
                .find(|environment| &environment._id == id)
                .map(|environment| environment.name.clone())
                .unwrap_or_else(|| id.clone())
        })
        .collect()
}
