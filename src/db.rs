use mongodb::{
    Client,
    bson::doc,
    options::{ClientOptions, ServerApi, ServerApiVersion},
};

pub async fn connect_db() -> mongodb::error::Result<Client> {
    let uri = "mongodb://localhost:27017/sharenv_test";

    let mut client_options = ClientOptions::parse(uri).await?;

    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();

    client_options.server_api = Some(server_api);

    let client = Client::with_options(client_options)?;

    client
        .database("sharenv_test")
        .run_command(doc! { "ping": 1 })
        .await?;

    println!("connected to MongoDB!");

    Ok(client)
}
