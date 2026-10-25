use crate::{
    models::{Environment, Project},
    settings::AppConfig,
    utils::env_file::{create_env_file, get_env_name},
};
use mongodb::{
    Client,
    bson::{doc, to_document},
};
use std::env;

pub async fn run(client: &Client, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let path = env::current_dir()?;
    let cwd_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let collection = client
        .database(&config.mongodb.database)
        .collection::<mongodb::bson::Document>("projects");

    if collection
        .find_one(doc! { "name": &cwd_name })
        .await?
        .is_some()
    {
        return Err(format!("Project '{cwd_name}' already exists in the database").into());
    }

    let mut environments = Vec::new();

    for file in &config.environment_files.files {
        create_env_file(file)?;

        let env_name = get_env_name(file)?;

        environments.push(Environment {
            name: env_name,
            variables: vec![],
        });
    }

    let project = Project {
        name: cwd_name.clone(),
        environments,
    };

    collection.insert_one(to_document(&project)?).await?;

    println!("Project '{cwd_name}' initialized successfully!");

    Ok(())
}
