use crate::settings::AppConfig;

pub fn run(config: &AppConfig) {
    let name = env!("CARGO_PKG_NAME");

    println!("{name} - Manage your project environments\n");
    println!("USAGE:");
    println!("    {name} <COMMAND> [OPTIONS]\n");
    println!("    using config v{} \n", &config.config_version);
    println!("COMMANDS:");
    println!("    register, r   Register a new account");
    println!("    login, l      User login");
    println!("    logout        Revoke your server-side session and remove local tokens");
    println!("    verify        Verify your account with the emailed code [email]");
    println!("    resend        Resend the verification code [email]");
    println!("    init, i       Initialize a project [-p|--project <project_name>] [--adopt]");
    println!("    invite        Invite a user to the current project <email> [--role <writer|reader>]");
    println!("    invites       Manage invitations: invites list | invites cancel <id> | invites accept <token>");
    println!("    add, a        Create an environment [-e|--environment <env_name>]");
    println!("    push          Push variables [-e|--environment <env_name>] [--show-values]");
    println!("                  Omit -e to push all environments with local .env files");
    println!("    pull          Pull variables [-e|--environment <env_name>] [--show-values]");
    println!("                  Omit -e to pull all environments");
    println!("                  Use [-pid|--project_id <project_id>] to pull a shared project");
    println!("    list, ls      List your projects or the current project's environments");
    println!("                  Usage: list projects | list environments");
    println!(
        "    delete, d     Delete the current project, a project by name, or an environment [-y|--yes]"
    );
    println!(
        "                  Usage: delete | delete -p|--project <project_name> | delete -e|--environment <env_name>"
    );
    println!("                  Project deletion requires your password (server re-authentication)");
    println!("    help, h       Display this help message");
    println!();
    println!("GLOBAL OPTIONS:");
    println!("    --insecure    Allow plaintext HTTP to a non-local host (not recommended)");
    println!();
    println!("OUTPUT:");
    println!("    Variable values are hidden by default. Pass --show-values to reveal");
    println!("    sanitized values when pushing or pulling.");
}
