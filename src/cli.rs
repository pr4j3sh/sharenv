use crate::api::client::ApiClient;
use crate::{
    commands,
    settings::AppConfig,
    utils::args::{parse_switch, take_cacert, take_profile},
};

pub async fn run_cli(config: &mut AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = std::env::args().collect();

    // `--insecure`, `--profile` and `--cacert` are global, per-invocation
    // switches. They are applied here and deliberately never read from
    // config or environment variables.
    let (remaining, insecure) = parse_switch(args.get(2..).unwrap_or(&[]), &["--insecure"]);
    let (remaining, profile) = take_profile(&remaining)?;
    let (remaining, cacert) = take_cacert(&remaining)?;
    args.truncate(2);
    args.extend(remaining);
    config.api_config.insecure = insecure;
    config.api_config.cacert = cacert;
    config.profile = profile;

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
        "whoami" => commands::whoami::run(config).await?,
        "init" | "i" => {
            let (init_args, adopt) = parse_switch(args.get(2..).unwrap_or(&[]), &["--adopt"]);
            commands::init::run(config, init_args, adopt).await?
        }
        "add" | "a" => commands::add::run(config, args.get(2..).unwrap_or(&[])).await?,
        "invite" => commands::invite::run(config, args.get(2..).unwrap_or(&[])).await?,
        "invites" => commands::invites::run(config, args.get(2..).unwrap_or(&[])).await?,
        "members" => commands::members::run(config, args.get(2..).unwrap_or(&[])).await?,
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
