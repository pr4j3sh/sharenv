use std::error::Error;

use crate::{settings::AppConfig, utils::session, utils::token};

pub async fn run(config: &AppConfig) -> Result<(), Box<dyn Error>> {
    let server_result = match session::authenticated_client(config).await {
        Ok(client) => client.logout().await,
        Err(error) => {
            // Cannot refresh/build an authenticated client (e.g. never logged
            // in, or offline). Still clear whatever tokens we hold locally.
            token::clear_session(&config.auth)?;
            eprintln!("warning: {error}");
            eprintln!("cleared the local session; the server may not have been notified");
            return Ok(());
        }
    };

    let cleared = token::clear_session(&config.auth);

    match (server_result, cleared) {
        (Ok(_), Ok(())) => {
            println!("logged out and revoked your server-side session.");
            Ok(())
        }
        (Err(error), Ok(())) => {
            eprintln!("warning: cleared the local session, but server-side revocation failed: {error}");
            eprintln!("your token may remain valid until it expires");
            Ok(())
        }
        (Ok(_), Err(error)) => Err(format!(
            "logged out on the server, but failed to clear local tokens: {error}"
        )
        .into()),
        (Err(server_error), Err(clear_error)) => Err(format!(
            "server-side revocation failed ({server_error}) and local token removal failed ({clear_error}); your token may remain valid until it expires"
        )
        .into()),
    }
}
