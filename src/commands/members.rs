use std::error::Error;

use zeroize::Zeroize;

use crate::{
    settings::AppConfig,
    utils::{
        env_select::{environment_names_for, print_environment_list, prompt_selection},
        local_config,
        output::{print_table, sanitize},
        prompt::prompt_password,
        session,
        validation::validate_email,
    },
};

pub async fn run(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    match args.first().map(String::as_str).unwrap_or("") {
        "role" => role(config, args.get(1..).unwrap_or(&[])).await,
        "environment" => envs(config, args.get(1..).unwrap_or(&[])).await,
        "" => Err(format!(
            "missing subject; use '{} list members' to view members, or '{} members role <email> <writer|reader>' / '{} members environment <email>' to change them",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_NAME")
        )
        .into()),
        other => Err(format!(
            "'{other}' is not a valid members subject; use 'role <email> <writer|reader>' or 'environment <email>'. To view members, use '{0} list members'",
            env!("CARGO_PKG_NAME")
        )
        .into()),
    }
}

pub async fn list_impl(config: &AppConfig) -> Result<(), Box<dyn Error>> {
    let json = config.json_output;
    let project_id = local_config::project_id(config)?;
    let client = session::authenticated_client(config).await?;

    let environments = client.get_environments(&project_id, None).await?;
    let members = client.list_members(&project_id).await?;

    if json {
        let items: Vec<serde_json::Value> = members
            .iter()
            .map(|member| {
                let grants = environment_names_for(&member.environments, &environments);
                let email = if member.email.is_empty() {
                    format!("<unknown: {}>", member.user_id)
                } else {
                    member.email.clone()
                };

                serde_json::json!({
                    "email": email,
                    "role": member.role,
                    "environments": grants,
                })
            })
            .collect();

        crate::utils::output::json(&items);
        return Ok(());
    }

    let rows = members
        .iter()
        .map(|member| {
            let grants = environment_names_for(&member.environments, &environments).join(", ");
            let email = if member.email.is_empty() {
                format!("<unknown: {}>", member.user_id)
            } else {
                sanitize(&member.email)
            };

            vec![email, member.role.clone(), grants]
        })
        .collect::<Vec<_>>();

    print_table(&["EMAIL", "ROLE", "GRANTS"], &rows);

    Ok(())
}

async fn role(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    if args.len() != 2 {
        return Err("usage: members role <email> <writer|reader>".into());
    }

    let email = args[0].trim().to_lowercase();
    validate_email(&email)?;

    let role = args[1].to_ascii_lowercase();

    if role != "writer" && role != "reader" {
        return Err("role must be 'writer' or 'reader'".into());
    }

    let project_id = local_config::project_id(config)?;
    let client = session::authenticated_client(config).await?;
    let environments = client.get_environments(&project_id, None).await?;
    let members = client.list_members(&project_id).await?;
    let member = find_member(&members, &email)?;

    if member.role == "owner" {
        return Err("cannot change the project owner's role".into());
    }

    let mut password = prompt_password("your password: ")?;
    let updated = client
        .update_member_access(
            &project_id,
            &member.user_id,
            member.environments.clone(),
            &role,
            &password,
        )
        .await?;
    password.zeroize();

    let grants = environment_names_for(&updated.environments, &environments).join(", ");

    println!(
        "Updated '{}' to role '{}' (grants unchanged: {}).",
        sanitize(&email),
        updated.role,
        grants
    );

    Ok(())
}

async fn envs(config: &AppConfig, args: &[String]) -> Result<(), Box<dyn Error>> {
    if args.len() != 1 {
        return Err("usage: members environment <email>".into());
    }

    let email = args[0].trim().to_lowercase();
    validate_email(&email)?;

    let project_id = local_config::project_id(config)?;
    let client = session::authenticated_client(config).await?;
    let environments = client.get_environments(&project_id, None).await?;
    let members = client.list_members(&project_id).await?;
    let member = find_member(&members, &email)?;

    if member.role == "owner" {
        return Err("cannot change the project owner's grants".into());
    }

    print_environment_list(&environments);

    let granted = prompt_selection(
        &environments,
        &format!(
            "grant access to which environments for '{email}'? (names, 'all') [all]: "
        ),
    )?;

    let mut password = prompt_password("your password: ")?;
    let role = member.role.clone();
    let updated = client
        .update_member_access(
            &project_id,
            &member.user_id,
            granted,
            &role,
            &password,
        )
        .await?;
    password.zeroize();

    let grants = environment_names_for(&updated.environments, &environments).join(", ");

    println!(
        "Updated grants for '{}' to: {grants}.",
        sanitize(&email)
    );

    Ok(())
}

fn find_member<'a>(
    members: &'a [crate::models::MemberModel],
    email: &str,
) -> Result<&'a crate::models::MemberModel, Box<dyn Error>> {
    members
        .iter()
        .find(|member| member.email == email)
        .ok_or_else(|| format!("'{email}' is not a member of this project").into())
}
