mod api;
mod cli;
mod commands;
mod models;
mod settings;
mod utils;
use settings::AppConfig;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load()?;

    cli::run_cli(&config).await?;

    Ok(())
}
