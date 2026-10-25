use crate::settings::AppConfig;
use mongodb::{
    Client,
    bson::doc,
    options::{ClientOptions, ServerApi, ServerApiVersion},
};

pub async fn connect_db(config: &AppConfig) -> mongodb::error::Result<Client> {
    let uri = &config.mongodb.uri;

    let mut client_options = ClientOptions::parse(uri).await?;

    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();

    client_options.server_api = Some(server_api);

    let client = Client::with_options(client_options)?;

    client
        .database("admin")
        .run_command(doc! { "ping": 1 })
        .await?;

    println!("connected to MongoDB!");

    Ok(client)
}
