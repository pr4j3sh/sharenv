use crate::api::client::ApiClient;
use crate::{commands, settings::AppConfig, utils::args::parse_name_arg};

pub async fn run_cli(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("");

    match cmd {
        "register" | "r" => {
            let client = ApiClient::new(&config.api_config);
            commands::register::run(&client, config).await?
        }
        "login" | "l" => {
            let client = ApiClient::new(&config.api_config);
            commands::login::run(&client, config).await?
        }
        "init" | "i" => {
            let name = parse_name_arg(args.get(2..).unwrap_or(&[]))?;
            commands::init::run(config, name).await?
        }
        "add" | "a" => commands::add::run(config, args.get(2..).unwrap_or(&[])).await?,
        "push" => commands::push::run(config, args.get(2..).unwrap_or(&[])).await?,
        "pull" => commands::pull::run(config, args.get(2..).unwrap_or(&[])).await?,
        "list" | "ls" => commands::list::run(config, args.get(2..).unwrap_or(&[])).await?,
        "help" | "h" | "--help" | "-h" => commands::help::run(config),

        "" => println!(
            "Please provide a command. Run '{} --help' for usage.",
            env!("CARGO_PKG_NAME")
        ),
        _ => println!(
            "'{}' is not a valid command. Run '{} --help' for usage.",
            cmd,
            env!("CARGO_PKG_NAME")
        ),
    }

    Ok(())
}
