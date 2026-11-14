use std::error::Error;
use std::path::Path;

use zeroize::Zeroize;

use crate::{
    api::client::ApiClient,
    settings::AppConfig,
    utils::{
        env_select::environment_names_for,
        local_config,
        output::sanitize,
        prompt::prompt_password,
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
    let project_id = local_config::project_id(config)?;

    let client = session::authenticated_client(config).await?;
    let environments = client.get_environments(&project_id, None).await?;
    let invitations = client.list_invitations(&project_id).await?;

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
    let project_id = local_config::project_id(config)?;

    let mut password = prompt_password("your password: ")?;

    let client = session::authenticated_client(config).await?;
    client
        .cancel_invitation(&project_id, invitation_id, &password)
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

    // Link this directory to the project, like `init` does, when it is not
    // already bound elsewhere. Never overwrite an existing binding.
    let linked = link_cwd(config, &client, &accepted.project_id).await?;

    if linked {
        println!("Pull it with: {} pull <environment>", env!("CARGO_PKG_NAME"));
    } else {
        println!(
            "Pull it with: {} pull --project-id {} <environment>",
            env!("CARGO_PKG_NAME"),
            accepted.project_id
        );
    }

    Ok(())
}

/// Record the accepted project in `./.sharenv/config.toml` so `push`, `pull`
/// and friends work here without `--project-id`. Returns false (leaving any
/// existing config untouched) when the directory cannot be linked.
async fn link_cwd(
    config: &AppConfig,
    client: &ApiClient,
    project_id: &str,
) -> Result<bool, Box<dyn Error>> {
    let existing = local_config::load(config).ok();
    let bound = existing
        .as_ref()
        .is_some_and(|local| !local.project.id.is_empty());

    if bound {
        let existing_id = existing.map(|local| local.project.id).unwrap_or_default();
        println!(
            "{} already records project '{}'; leaving it untouched.",
            config.config_file_path,
            existing_id
        );
        return Ok(false);
    }

    if existing.is_none() && Path::new(&config.config_file_path).exists() {
        // A config file exists but cannot be used (unreadable or corrupt);
        // never overwrite it.
        println!(
            "warning: {} exists but could not be read; leaving it untouched.",
            config.config_file_path
        );
        return Ok(false);
    }

    // Fetch the project so the binding records real identity metadata (the
    // same metadata `init --adopt` records); without it the binding could
    // misname the project and break the push guard.
    let project = match client.get_projects().await {
        Ok(projects) => projects
            .into_iter()
            .find(|project| project._id == project_id),
        Err(error) => {
            println!(
                "warning: joined the project, but could not fetch its details ({error}); directory not linked."
            );
            return Ok(false);
        }
    };

    let Some(project) = project else {
        println!(
            "warning: joined the project, but could not find it in the project list; directory not linked."
        );
        return Ok(false);
    };

    local_config::save_identity(
        config,
        &project._id,
        &project.name,
        Some(&project.user_id),
        local_config::CREATED_BY_INVITES,
    )?;

    println!(
        "Linked this directory to project '{}' (owner: {}) ({})",
        sanitize(&project.name),
        sanitize(&project.user_id),
        project._id
    );

    Ok(true)
}
