//! Project identity checks shared by `push` and `pull`.
//!
//! The client cannot prove who owns a project; the server does that. This
//! guard makes the destination visible before secrets move and refuses to
//! continue when local metadata disagrees with the server.

use std::error::Error;

use crate::{
    api::client::ApiClient,
    models::ProjectModel,
    settings::AppConfig,
    utils::{local_config, output::{line, sanitize}, prompt},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Push,
    Pull,
}

/// Resolve the project from the server's project list and make the user
/// confirm transfers that are not clearly safe.
///
/// `GET /project/` only returns projects the current user owns or belongs to,
/// so a missing project means "not a member" and is always refused.
pub async fn authorize(
    config: &AppConfig,
    client: &ApiClient,
    current_user: Option<&str>,
    project_id: &str,
    access: Access,
) -> Result<Option<ProjectModel>, Box<dyn Error>> {
    let projects = client.get_projects().await?;

    let project = projects
        .into_iter()
        .find(|project| project._id == project_id)
        .ok_or_else(|| {
            format!(
                "you are not a member of project '{}' (or the project does not exist)",
                sanitize(project_id)
            )
        })?;

    let is_owner = current_user.is_some_and(|user| user == project.user_id);
    let user_known = current_user.is_some();
    line(
        config.json_output,
        format!(
            "Project: {} (owner: {})",
            sanitize(&project.name),
            owner_label(&project, is_owner, user_known)
        ),
    );

    let identity = check_local_identity(config, &project);
    let mut needs_confirmation = !is_owner;

    for warning in &identity.warnings {
        eprintln!("warning: {warning}");
        needs_confirmation = true;
    }

    if access == Access::Push && !identity.mismatches.is_empty() {
        return Err(format!(
            "refusing to push to '{}': {}",
            sanitize(&project.name),
            identity.mismatches.join("; ")
        )
        .into());
    }

    for mismatch in &identity.mismatches {
        eprintln!("warning: {mismatch}");
        needs_confirmation = true;
    }

    if access == Access::Pull && !is_owner {
        if user_known {
            eprintln!(
                "warning: project '{}' is owned by '{}', not you",
                sanitize(&project.name),
                sanitize(&project.user_id)
            );
        } else {
            eprintln!(
                "warning: could not verify project ownership; owner is '{}'",
                sanitize(&project.user_id)
            );
        }
    }

    if needs_confirmation {
        let prompt_text = match access {
            Access::Push => format!(
                "Push to project '{}' (owner: {})? [y/N]: ",
                sanitize(&project.name),
                owner_label(&project, is_owner, user_known)
            ),
            Access::Pull => format!(
                "Pull project '{}' (owner: {})? [y/N]: ",
                sanitize(&project.name),
                owner_label(&project, is_owner, user_known)
            ),
        };

        if !prompt::confirm(&prompt_text)? {
            line(config.json_output, "Cancelled.");
            return Ok(None);
        }
    }

    Ok(Some(project))
}

#[derive(Default)]
struct IdentityCheck {
    warnings: Vec<String>,
    mismatches: Vec<String>,
}

fn check_local_identity(config: &AppConfig, project: &ProjectModel) -> IdentityCheck {
    let mut check = IdentityCheck::default();

    let Ok(local) = local_config::load(config) else {
        return check;
    };

    // A `pull --project-id` may target a project other than the one recorded locally.
    if local.project.id.is_empty() || local.project.id != project._id {
        return check;
    }

    if local.project.created_by != local_config::CREATED_BY_INIT {
        check.warnings.push(format!(
            "{} has no identity metadata; run '{} init --adopt' to record it",
            config.config_file_path,
            env!("CARGO_PKG_NAME")
        ));
    }

    if !local.project.name.is_empty() && local.project.name != project.name {
        check.mismatches.push(format!(
            "local config names project '{}' but the server reports '{}'",
            sanitize(&local.project.name),
            sanitize(&project.name)
        ));
    }

    if !local.project.owner_id.is_empty() && local.project.owner_id != project.user_id {
        check.mismatches.push(format!(
            "local config records owner '{}' but the server reports '{}'",
            sanitize(&local.project.owner_id),
            sanitize(&project.user_id)
        ));
    }

    if !local.project.base_url.is_empty()
        && local.project.base_url != config.api_config.base_url.trim_end_matches('/')
    {
        check.warnings.push(format!(
            "API base URL changed from '{}' to '{}'",
            sanitize(&local.project.base_url),
            sanitize(config.api_config.base_url.trim_end_matches('/'))
        ));
    }

    check
}

pub fn owner_label(project: &ProjectModel, is_owner: bool, user_known: bool) -> String {
    if is_owner {
        "you".to_string()
    } else if user_known {
        format!("{} (not you)", sanitize(&project.user_id))
    } else {
        format!("{} (unverified)", sanitize(&project.user_id))
    }
}
