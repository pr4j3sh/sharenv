//! Authenticated API client construction shared by all commands.
//!
//! Access tokens live for a short time (the server issues ~15 minute
//! tokens), so before a command runs we transparently refresh the session
//! when the token is expired or about to expire. If the refresh fails we
//! clear the local session and tell the user to log in again.
//!
//! Known limitation: a single interactive command open longer than the
//! token lifetime can still see a 401 mid-run (e.g. while sitting at a
//! prompt). Those errors surface with a clear "run 'sharenv login'" hint.

use crate::{
    api::client::ApiClient,
    settings::AppConfig,
    utils::{jwt, token},
};

/// Build an API client for an authenticated command, refreshing the access
/// token first when it is expired or expires within the next 60 seconds.
pub async fn authenticated_client(
    config: &AppConfig,
) -> Result<ApiClient, Box<dyn std::error::Error>> {
    let token = token::get_token(&config.auth)?.ok_or_else(|| {
        format!(
            "not logged in; run '{} login' first",
            env!("CARGO_PKG_NAME")
        )
    })?;

    let token = if jwt::expiring_within(&token, 60) {
        refresh_session(config).await?
    } else {
        token
    };

    Ok(ApiClient::new(&config.api_config)?.with_token(token))
}

/// The authenticated user's id, read unverified from the token.
pub fn current_user(client: &ApiClient) -> Option<String> {
    client.token.as_deref().and_then(jwt::subject)
}

async fn refresh_session(config: &AppConfig) -> Result<String, Box<dyn std::error::Error>> {
    let refresh_token = token::get_refresh_token(&config.auth)?.ok_or_else(|| {
        format!(
            "your session expired and no refresh token is stored; run '{} login' again",
            env!("CARGO_PKG_NAME")
        )
    })?;

    let client = ApiClient::new(&config.api_config)?;
    let tokens = match client.refresh(refresh_token).await {
        Ok(tokens) => tokens,
        Err(_) => {
            // The refresh token is dead too; drop whatever remains locally so
            // the user starts from a clean slate. The server does not need to
            // be told: an invalid refresh token cannot be revoked anyway.
            let _ = token::delete_session_tokens(&config.auth);
            return Err(format!(
                "your saved session expired or was revoked; run '{} login' again",
                env!("CARGO_PKG_NAME")
            )
            .into());
        }
    };

    token::save_session(&config.auth, &tokens.token, &tokens.refresh_token).map_err(
        |error| format!("refreshed session but failed to save the new tokens: {error}"),
    )?;

    Ok(tokens.token)
}
