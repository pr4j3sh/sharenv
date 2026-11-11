use keyring::Entry;

use crate::settings::AuthConfig;

pub fn save_token(auth: &AuthConfig, token: &str) -> keyring::Result<()> {
    let entry = Entry::new(&auth.token_service, &auth.token_account)?;
    entry.set_password(token)
}

pub fn get_token(auth: &AuthConfig) -> keyring::Result<Option<String>> {
    let entry = Entry::new(&auth.token_service, &auth.token_account)?;

    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn save_refresh_token(auth: &AuthConfig, token: &str) -> keyring::Result<()> {
    let entry = Entry::new(&auth.token_service, &refresh_account(auth))?;
    entry.set_password(token)
}

pub fn get_refresh_token(auth: &AuthConfig) -> keyring::Result<Option<String>> {
    let entry = Entry::new(&auth.token_service, &refresh_account(auth))?;

    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error),
    }
}

/// Save both halves of an authenticated session.
pub fn save_session(auth: &AuthConfig, token: &str, refresh_token: &str) -> keyring::Result<()> {
    save_token(auth, token)?;
    save_refresh_token(auth, refresh_token)
}

/// Delete the access and refresh tokens. Pending verification state is left
/// alone; use `clear_session` when the user explicitly logs out.
pub fn delete_session_tokens(auth: &AuthConfig) -> keyring::Result<()> {
    delete_token(auth)?;
    delete_account(auth, &refresh_account(auth))
}

/// Delete everything tied to the account: access/refresh tokens and any
/// pending verification marker.
pub fn clear_session(auth: &AuthConfig) -> keyring::Result<()> {
    delete_session_tokens(auth)?;
    delete_account(auth, &pending_verify_account(auth))
}

/// Record the email of an account awaiting OTP verification. This lets
/// `verify`/`resend` work without re-entering it.
pub fn save_pending_verify(auth: &AuthConfig, email: &str) -> keyring::Result<()> {
    let entry = Entry::new(&auth.token_service, &pending_verify_account(auth))?;
    entry.set_password(&email.trim().to_lowercase())
}

pub fn get_pending_verify(auth: &AuthConfig) -> keyring::Result<Option<String>> {
    let entry = Entry::new(&auth.token_service, &pending_verify_account(auth))?;

    match entry.get_password() {
        // Pre-email versions stored the user id; those values are not valid
        // emails, so ignore them instead of prompting for a wrong address.
        Ok(value) if value.contains('@') => Ok(Some(value)),
        Ok(_) => Ok(None),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn clear_pending_verify(auth: &AuthConfig) -> keyring::Result<()> {
    delete_account(auth, &pending_verify_account(auth))
}

pub fn delete_token(auth: &AuthConfig) -> keyring::Result<()> {
    delete_account(auth, &auth.token_account)
}

fn delete_account(auth: &AuthConfig, account: &str) -> keyring::Result<()> {
    let entry = Entry::new(&auth.token_service, account)?;

    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(error),
    }
}

fn refresh_account(auth: &AuthConfig) -> String {
    format!("{}-refresh", auth.token_account)
}

fn pending_verify_account(auth: &AuthConfig) -> String {
    format!("{}-pending-verify", auth.token_account)
}
