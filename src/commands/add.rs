use mongodb::Client;

use crate::settings::AppConfig;

pub async fn run(_client: &Client, _config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("adding");
    Ok(())
}
