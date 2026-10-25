use crate::utils::env_parser::read_env_file;
use mongodb::{
    Client,
    bson::{doc, to_bson},
};
use std::env;

pub async fn run(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("pushing");

    let path = env::current_dir()?;
    let project_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let dev_variables = read_env_file(".env.dev")?;
    let prod_variables = read_env_file(".env.prod")?;

    let collection = client
        .database("sharenv_test")
        .collection::<mongodb::bson::Document>("projects");

    let project = collection
        .find_one(doc! { "name": &project_name })
        .await?
        .ok_or_else(|| format!("Project '{project_name}' does not exist in MongoDB"))?;

    let environments = project
        .get_array("environments")
        .map_err(|_| "Project has no valid environments array")?;

    for env_name in ["dev", "prod"] {
        let exists = environments.iter().any(|environment| {
            environment
                .as_document()
                .and_then(|environment| environment.get_str("name").ok())
                == Some(env_name)
        });

        if !exists {
            return Err(format!(
                "Environment '{env_name}' does not exist for project '{project_name}'"
            )
            .into());
        }
    }

    collection
        .update_one(
            doc! { "name": &project_name },
            doc! {
                "$set": {
                    "environments.$[dev].variables": to_bson(&dev_variables)?,
                    "environments.$[prod].variables": to_bson(&prod_variables)?,
                }
            },
        )
        .array_filters(vec![
            doc! { "dev.name": "dev" },
            doc! { "prod.name": "prod" },
        ])
        .await?;

    println!("Project '{project_name}' pushed successfully!");

    Ok(())
}
