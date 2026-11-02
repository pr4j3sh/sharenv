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

#[allow(dead_code)]
pub fn delete_token(auth: &AuthConfig) -> keyring::Result<()> {
    let entry = Entry::new(&auth.token_service, &auth.token_account)?;

    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(error),
    }
}
