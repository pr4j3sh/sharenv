use crate::{commands, settings::AppConfig};
use mongodb::Client;

pub async fn run_cli(
    client: &Client,
    config: &AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("");

    match cmd {
        "init" | "i" => commands::init::run(client, config).await?,
        "add" | "a" => commands::add::run(client, config).await?,
        "push" => commands::push::run(client, config).await?,
        "pull" => commands::pull::run(client, config).await?,

        "help" | "h" => {
            println!("sharenv - Manage your project environments\n");
            println!("USAGE:");
            println!("    sharenv <COMMAND>\n");
            println!("COMMANDS:");
            println!("    init, i       Initialize a project");
            println!("    add, a        Add an environment variable");
            println!("    push          Push environment variables to the database");
            println!("    pull          Pull environment variables from the database");
            println!("    help, h      Display this help message");
        }

        "" => println!("Please provide a command. Run 'sharenv --help' for usage."),
        _ => println!(
            "'{}' is not a valid command. Run 'sharenv --help' for usage.",
            cmd
        ),
    }

    Ok(())
}
