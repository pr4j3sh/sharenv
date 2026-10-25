mod cli;
mod commands;
mod db;
mod models;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = db::connect_db().await?;

    cli::run_cli(&client).await?;

    Ok(())
}
