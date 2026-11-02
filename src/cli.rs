use crate::{
    api::client::ApiClient,
    commands,
    settings::AppConfig,
    utils::args::parse_name_arg,
};

pub async fn run_cli(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("");

    match cmd {
        "register" | "r" => {
            let client = ApiClient::new(config.api_config.base_url.clone());
            commands::register::run(&client, config).await?
        }
        "login" | "l" => {
            let client = ApiClient::new(config.api_config.base_url.clone());
            commands::login::run(&client, config).await?
        }
        "init" | "i" => {
            let name = parse_name_arg(args.get(2..).unwrap_or(&[]))?;
            commands::init::run(config, name).await?
        }
        "add" | "a" => commands::add::run(config).await?,
        "push" => commands::push::run(config).await?,
        "pull" => commands::pull::run(config).await?,

        "help" | "h" | "--help" | "-h" => {
            println!("sharenv - Manage your project environments\n");
            println!("USAGE:");
            println!("    sharenv <COMMAND>\n");
            println!("COMMANDS:");
            println!("    register, r   Register a new account");
            println!("    login, l      User login");
            println!("    init, i       Initialize a project [-n|--name <project_name>]");
            println!("    add, a        Add an environment variable (not implemented)");
            println!("    push          Push environment variables to the API (not implemented)");
            println!("    pull          Pull environment variables from the API (not implemented)");
            println!("    help, h       Display this help message");
        }

        "" => println!("Please provide a command. Run 'sharenv --help' for usage."),
        _ => println!(
            "'{}' is not a valid command. Run 'sharenv --help' for usage.",
            cmd
        ),
    }

    Ok(())
}
