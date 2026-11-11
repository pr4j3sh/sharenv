use std::error::Error;

use zeroize::Zeroize;

use crate::{
    settings::AppConfig,
    utils::{
        env_select::environment_names_for,
        local_config,
        output::sanitize,
        session,
    },
};

pub async fn run(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    match args.first().map(String::as_str).unwrap_or("") {
        "list" | "ls" => list(config).await,
        "cancel" => {
            let invitation_id = args.get(1).ok_or(
                "missing invitation id; usage: invites cancel <invitation_id>",
            )?;

            cancel(config, invitation_id).await
        }
        "accept" => {
            let token = args.get(1).ok_or("missing token; usage: invites accept <token>")?;

            accept(config, token).await
        }
        other => Err(format!(
            "'{other}' is not a valid invites subject; use 'list', 'cancel <id>' or 'accept <token>'"
        )
        .into()),
    }
}

async fn list(config: &AppConfig) -> Result<(), Box<dyn Error>> {
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
    let invitations = client.list_invitations(&local.project.id).await?;

    if invitations.is_empty() {
        println!("No pending invitations.");
        return Ok(());
    }

    for invitation in invitations {
        let status = if invitation.accepted_at.is_some() {
            "accepted"
        } else {
            "pending"
        };

        let granted = environment_names_for(&invitation.environments, &environments);
        let id = invitation._id.as_deref().unwrap_or("<unknown>");

        println!("{id}");
        println!("  email:        {}", sanitize(&invitation.email));
        println!("  role:         {}", invitation.role);
        println!("  environments: {}", granted.join(", "));
        println!("  status:       {status}");
        println!("  expires:      {}", invitation.expires_at.to_rfc3339());
    }

    Ok(())
}

async fn cancel(config: &AppConfig, invitation_id: &str) -> Result<(), Box<dyn Error>> {
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

    let mut password = rpassword::prompt_password("your password: ")?;

    if password.is_empty() {
        password.zeroize();
        return Err("password cannot be empty".into());
    }

    let client = session::authenticated_client(config).await?;
    client
        .cancel_invitation(&local.project.id, invitation_id, &password)
        .await?;
    password.zeroize();

    println!("Cancelled invitation '{}'.", sanitize(invitation_id));

    Ok(())
}

async fn accept(config: &AppConfig, token_value: &str) -> Result<(), Box<dyn Error>> {
    let client = session::authenticated_client(config).await?;
    let accepted = client.accept_invitation(token_value).await?;

    println!(
        "Joined project '{}' (role: {}, environments: {} granted).",
        sanitize(&accepted.project_id),
        accepted.role,
        accepted.environments.len()
    );
    println!(
        "Pull it with: {} pull --project-id {} <environment>",
        env!("CARGO_PKG_NAME"),
        accepted.project_id
    );

    Ok(())
}


