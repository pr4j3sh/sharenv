use crate::api::client::ApiClient;
use crate::{
    commands,
    settings::AppConfig,
    utils::args::{parse_project_arg, parse_switch},
};

pub async fn run_cli(config: &mut AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = std::env::args().collect();

    // `--insecure` is a global, per-invocation switch. It is applied here and
    // deliberately never read from config or environment variables.
    let (remaining, insecure) = parse_switch(args.get(2..).unwrap_or(&[]), &["--insecure"]);
    args.truncate(2);
    args.extend(remaining);
    config.api_config.insecure = insecure;

    let cmd = args.get(1).map(String::as_str).unwrap_or("");

    match cmd {
        "register" | "r" => {
            let client = ApiClient::new(&config.api_config)?;
            commands::register::run(&client, config).await?
        }
        "login" | "l" => {
            let client = ApiClient::new(&config.api_config)?;
            commands::login::run(&client, config).await?
        }
        "logout" => commands::logout::run(config).await?,
        "verify" => commands::verify::run(config, args.get(2..).unwrap_or(&[])).await?,
        "resend" => commands::resend::run(config, args.get(2..).unwrap_or(&[])).await?,
        "init" | "i" => {
            let (init_args, adopt) = parse_switch(args.get(2..).unwrap_or(&[]), &["--adopt"]);
            let name = parse_project_arg(&init_args)?;
            commands::init::run(config, name, adopt).await?
        }
        "add" | "a" => commands::add::run(config, args.get(2..).unwrap_or(&[])).await?,
        "invite" => commands::invite::run(config, args.get(2..).unwrap_or(&[])).await?,
        "invites" => commands::invites::run(config, args.get(2..).unwrap_or(&[])).await?,
        "push" => commands::push::run(config, args.get(2..).unwrap_or(&[])).await?,
        "pull" => commands::pull::run(config, args.get(2..).unwrap_or(&[])).await?,
        "list" | "ls" => commands::list::run(config, args.get(2..).unwrap_or(&[])).await?,
        "delete" | "d" => commands::delete::run(config, args.get(2..).unwrap_or(&[])).await?,
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
