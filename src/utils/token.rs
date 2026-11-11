use keyring::Entry;

use crate::settings::AuthConfig;

/// Where credentials live in the OS keyring.
///
/// The base slot is derived from the API base URL so that sessions against
/// different servers (e.g. local dev vs production) never overwrite each
/// other. An explicit `--profile <name>` on the command line overrides the
/// derivation for that invocation.
pub fn account_for(auth: &AuthConfig, base_url: &str, profile: Option<&str>) -> String {
    match profile {
        Some(profile) => format!("{}-{}", auth.token_account, sanitize_profile(profile)),
        None => format!("{}-{:016x}", auth.token_account, fnv1a(&normalize(base_url))),
    }
}

fn normalize(base_url: &str) -> String {
    base_url.trim().trim_end_matches('/').to_lowercase()
}

/// FNV-1a, pinned so slots stay stable across Rust versions.
fn fnv1a(input: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;

    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    hash
}

fn sanitize_profile(profile: &str) -> String {
    profile
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-' || *c == '_')
        .collect()
}

/// Validate a `--profile` value; returns it normalized (lowercase) or a plain
/// error describing what's allowed.
pub fn validate_profile(profile: &str) -> Result<String, Box<dyn std::error::Error>> {
    let normalized = profile.trim().to_lowercase();

    let valid = !normalized.is_empty()
        && normalized.len() <= 32
        && normalized
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');

    if !valid {
        return Err(format!(
            "invalid profile name '{profile}'; use 1-32 lowercase letters, digits, '-' or '_'"
        )
        .into());
    }

    Ok(normalized)
}

pub fn save_token(
    auth: &AuthConfig,
    base_url: &str,
    profile: Option<&str>,
    token: &str,
) -> keyring::Result<()> {
    let entry = Entry::new(&auth.token_service, &account_for(auth, base_url, profile))?;
    entry.set_password(token)
}

pub fn get_token(
    auth: &AuthConfig,
    base_url: &str,
    profile: Option<&str>,
) -> keyring::Result<Option<String>> {
    let entry = Entry::new(&auth.token_service, &account_for(auth, base_url, profile))?;

    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn save_refresh_token(
    auth: &AuthConfig,
    base_url: &str,
    profile: Option<&str>,
    token: &str,
) -> keyring::Result<()> {
    let entry = Entry::new(&auth.token_service, &refresh_account(auth, base_url, profile))?;
    entry.set_password(token)
}

pub fn get_refresh_token(
    auth: &AuthConfig,
    base_url: &str,
    profile: Option<&str>,
) -> keyring::Result<Option<String>> {
    let entry = Entry::new(&auth.token_service, &refresh_account(auth, base_url, profile))?;

    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error),
    }
}

/// Save both halves of an authenticated session.
#[allow(clippy::too_many_arguments)]
pub fn save_session(
    auth: &AuthConfig,
    base_url: &str,
    profile: Option<&str>,
    token: &str,
    refresh_token: &str,
) -> keyring::Result<()> {
    save_token(auth, base_url, profile, token)?;
    save_refresh_token(auth, base_url, profile, refresh_token)
}

/// Delete the access and refresh tokens. Pending verification state is left
/// alone; use `clear_session` when the user explicitly logs out.
pub fn delete_session_tokens(
    auth: &AuthConfig,
    base_url: &str,
    profile: Option<&str>,
) -> keyring::Result<()> {
    delete_account(auth, &account_for(auth, base_url, profile))?;
    delete_account(auth, &refresh_account(auth, base_url, profile))
}

/// Delete everything tied to the session: access/refresh tokens and any
/// pending verification marker.
pub fn clear_session(
    auth: &AuthConfig,
    base_url: &str,
    profile: Option<&str>,
) -> keyring::Result<()> {
    delete_session_tokens(auth, base_url, profile)?;
    delete_account(auth, &pending_verify_account(auth, base_url, profile))
}

/// Record the email of an account awaiting OTP verification. This lets
/// `verify`/`resend` work without re-entering it.
pub fn save_pending_verify(
    auth: &AuthConfig,
    base_url: &str,
    profile: Option<&str>,
    email: &str,
) -> keyring::Result<()> {
    let entry = Entry::new(
        &auth.token_service,
        &pending_verify_account(auth, base_url, profile),
    )?;
    entry.set_password(&email.trim().to_lowercase())
}

pub fn get_pending_verify(
    auth: &AuthConfig,
    base_url: &str,
    profile: Option<&str>,
) -> keyring::Result<Option<String>> {
    let entry = Entry::new(
        &auth.token_service,
        &pending_verify_account(auth, base_url, profile),
    )?;

    match entry.get_password() {
        Ok(value) if value.contains('@') => Ok(Some(value)),
        Ok(_) => Ok(None),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn clear_pending_verify(
    auth: &AuthConfig,
    base_url: &str,
    profile: Option<&str>,
) -> keyring::Result<()> {
    delete_account(auth, &pending_verify_account(auth, base_url, profile))
}

fn delete_account(auth: &AuthConfig, account: &str) -> keyring::Result<()> {
    let entry = Entry::new(&auth.token_service, account)?;

    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(error),
    }
}

fn refresh_account(auth: &AuthConfig, base_url: &str, profile: Option<&str>) -> String {
    format!("{}-refresh", account_for(auth, base_url, profile))
}

fn pending_verify_account(auth: &AuthConfig, base_url: &str, profile: Option<&str>) -> String {
    format!("{}-pending-verify", account_for(auth, base_url, profile))
}
