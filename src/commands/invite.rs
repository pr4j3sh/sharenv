use std::error::Error;

use zeroize::Zeroize;

use crate::{
    settings::AppConfig,
    utils::{
        env_select::{environment_names_for, print_environment_list, prompt_selection},
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

    let role = resolve_role(role_flag)?;

    let local =
        local_config::load(config).map_err(|error| format!("failed to load {}: {error}", config.config_file_path))?;

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

    print_environment_list(&environments);

    let granted = prompt_selection(
        &environments,
        "grant access to which environments? (names, 'all') [all]: ",
    )?;

    let mut password = prompt_password("your password (required to share): ")?;

    let invitation = client
        .create_invitation(
            &local.project.id,
            email.to_string(),
            &role,
            granted,
            &password,
        )
        .await?;

    password.zeroize();

    let granted_names = environment_names_for(&invitation.environments, &environments);

    println!(
        "Invitation created for '{}' (role: {}, environments: {}).",
        sanitize(&invitation.email),
        invitation.role,
        granted_names.join(", ")
    );
    println!("Invitation id: {}", invitation.invitation_id);

    if invitation.email_sent {
        println!(
            "An invitation email was sent to '{}'.",
            sanitize(&invitation.email)
        );
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

pub fn resolve_role(role_flag: Option<String>) -> Result<String, Box<dyn Error>> {
    match role_flag {
        Some(role) => {
            let role = role.to_ascii_lowercase();

            if role != "writer" && role != "reader" {
                return Err("--role must be 'writer' or 'reader'".into());
            }

            Ok(role)
        }
        None => {
            let input = prompt_line("role (writer/reader) [reader]: ")?;

            match input.to_ascii_lowercase().as_str() {
                "" | "reader" => Ok("reader".to_string()),
                "writer" => Ok("writer".to_string()),
                _ => Err("role must be 'writer' or 'reader'".into()),
            }
        }
    }
}

pub fn prompt_password(label: &str) -> Result<String, Box<dyn Error>> {
    let password = rpassword::prompt_password(label)?;

    if password.is_empty() {
        return Err("password cannot be empty".into());
    }

    Ok(password)
}

fn parse_args(args: &[String]) -> Result<(String, Option<String>), Box<dyn Error>> {
    let mut email: Option<String> = None;
    let mut role: Option<String> = None;
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--role" => {
                let value = iter.next().ok_or("missing value for --role")?;

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
