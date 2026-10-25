mod cli;
mod commands;
mod db;
mod models;
mod settings;
mod utils;
use settings::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load()?;
    let client = db::connect_db(&config).await?;

    cli::run_cli(&client, &config).await?;

    Ok(())
}
