use crate::models::{Environment, Project};
use crate::utils::env_file::create_env_file;
use mongodb::{
    Client,
    bson::{doc, to_document},
};
use std::env;

pub async fn run(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let path = env::current_dir()?;
    let cwd_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let collection = client
        .database("sharenv_test")
        .collection::<mongodb::bson::Document>("projects");

    if collection
        .find_one(doc! { "name": &cwd_name })
        .await?
        .is_some()
    {
        return Err(format!("Project '{cwd_name}' already exists in the database").into());
    }

    create_env_file(".env.dev")?;
    create_env_file(".env.prod")?;

    let project = Project {
        name: cwd_name.clone(),
        environments: vec![
            Environment {
                name: "dev".to_string(),
                variables: vec![],
            },
            Environment {
                name: "prod".to_string(),
                variables: vec![],
            },
        ],
    };

    collection.insert_one(to_document(&project)?).await?;

    println!("Project '{cwd_name}' initialized successfully!");

    Ok(())
}
