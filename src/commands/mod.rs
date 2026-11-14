pub mod add;
pub mod delete;
pub mod help;
pub mod init;
pub mod invite;
pub mod invites;
pub mod list;
pub mod login;
pub mod logout;
pub mod members;
pub mod pull;
pub mod push;
pub mod register;
pub mod resend;
pub mod verify;
pub mod whoami;

use std::error::Error;

use crate::settings::AppConfig;

/// Resolve the account email for `verify`/`resend`: an explicit argument,
/// else the pending-verification marker saved at registration, else a prompt.
pub(crate) fn resolve_email(
    config: &AppConfig,
    args: &[String],
) -> Result<String, Box<dyn Error>> {
    if let Some(email) = args.first() {
        let email = email.trim().to_lowercase();
        crate::utils::validation::validate_email(&email)?;
        return Ok(email);
    }

    let base_url = &config.api_config.base_url;
    let profile = config.profile.as_deref();

    if let Some(email) = crate::utils::token::get_pending_verify(&config.auth, base_url, profile)? {
        return Ok(email);
    }

    let input = crate::utils::prompt::prompt_line("email: ")?;
    let email = input.trim().to_lowercase();
    crate::utils::validation::validate_email(&email)?;

    Ok(email)
}
