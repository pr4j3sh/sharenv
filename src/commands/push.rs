use crate::{
    settings::AppConfig,
    utils::{
        env_file::get_env_name,
        env_parser::read_env_file,
        env_sync::{find_new_variables, get_existing_names},
    },
};
use mongodb::{
    Client,
    bson::{Document, doc, to_bson},
};
use std::{
    env,
    io::{self, Write},
};

pub async fn run(client: &Client, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("Pushing environment variables...");

    let path = env::current_dir()?;
    let project_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let collection = client
        .database(&config.mongodb.database)
        .collection::<Document>("projects");

    let project = collection
        .find_one(doc! { "name": &project_name })
        .await?
        .ok_or_else(|| format!("Project '{project_name}' does not exist in MongoDB"))?;

    let mut updates = Document::new();
    let mut array_filters = Vec::new();
    let mut has_new_variables = false;

    for (index, file) in config.environment_files.files.iter().enumerate() {
        let env_name = get_env_name(file)?;
        let variables = read_env_file(file)?;
        if variables.is_empty() {
            println!("Skipping empty environment file: {file}");
            continue;
        }
        let existing_names = get_existing_names(&project, &env_name)?;

        let new_variables = find_new_variables(&variables, &existing_names);

        if !new_variables.is_empty() {
            has_new_variables = true;

            for variable in new_variables {
                println!("  [{env_name}] {}", variable.name);
            }
        }

        let filter_name = format!("env{index}");

        updates.insert(
            format!("environments.$[{filter_name}].variables"),
            to_bson(&variables)?,
        );

        array_filters.push(doc! {
            format!("{filter_name}.name"): &env_name
        });
    }

    if has_new_variables {
        print!("\nPush these changes? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Push cancelled.");
            return Ok(());
        }
    } else {
        println!("No new variables found.");
    }

    collection
        .update_one(
            doc! { "_id": project.get_object_id("_id")? },
            doc! { "$set": updates },
        )
        .array_filters(array_filters)
        .await?;

    println!("Project '{project_name}' pushed successfully!");

    Ok(())
}
