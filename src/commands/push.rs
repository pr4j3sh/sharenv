use crate::utils::{
    env_parser::read_env_file,
    env_sync::{find_new_variables, get_existing_names},
};
use mongodb::{
    Client,
    bson::{Document, doc, to_bson},
};
use std::{
    env,
    io::{self, Write},
};

pub async fn run(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("Pushing environment variables...");

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
        .collection::<Document>("projects");

    let project = collection
        .find_one(doc! { "name": &project_name })
        .await?
        .ok_or_else(|| format!("Project '{project_name}' does not exist in MongoDB"))?;

    // Verify both environments exist and collect their current variable names.
    let dev_names = get_existing_names(&project, "dev")?;
    let prod_names = get_existing_names(&project, "prod")?;

    let new_dev = find_new_variables(&dev_variables, &dev_names);
    let new_prod = find_new_variables(&prod_variables, &prod_names);

    if new_dev.is_empty() && new_prod.is_empty() {
        println!("No new variables found.");
    } else {
        println!("\nNew variables to push:");

        for variable in &new_dev {
            println!("  [dev]  {}", variable.name);
        }

        for variable in &new_prod {
            println!("  [prod] {}", variable.name);
        }

        print!("\nPush these changes? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Push cancelled.");
            return Ok(());
        }
    }

    // Sync the environment variables with MongoDB.
    collection
        .update_one(
            doc! { "_id": project.get_object_id("_id")? },
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
