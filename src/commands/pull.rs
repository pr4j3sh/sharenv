use mongodb::Client;

pub async fn run(_client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("pulling");
    Ok(())
}
