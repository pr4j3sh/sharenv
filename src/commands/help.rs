use crate::settings::AppConfig;

pub fn run(config: &AppConfig) {
    let name = env!("CARGO_PKG_NAME");

    println!("{name} - Manage your project environments\n");
    println!("USAGE:");
    println!("    {name} <COMMAND> [ARGS] [OPTIONS]\n");
    println!("    using config v{} \n", &config.config_version);
    println!("COMMANDS:");
    println!("    register, r   Register a new account");
    println!("    login, l      User login");
    println!("    logout        Revoke your server-side session and remove local tokens");
    println!("    verify        Verify your account with the emailed code [email]");
    println!("    resend        Resend the verification code [email]");
    println!("    whoami        Show the logged-in user, session, API URL and keyring slot");
    println!("    init, i       Initialize a project [name] [--adopt]");
    println!("                  Bare name initializes with the current directory's name");
    println!("    add, a        Create an environment <environment>");
    println!("    push          Push variables [environment] [--show-values]");
    println!("                  Omit [environment] to push all environments with local .env files");
    println!("    pull          Pull variables [environment] [--show-values] [--project-id <project_id>]");
    println!("                  Omit [environment] to pull all environments");
    println!("                  Use --project-id to pull a shared project without a local .sharenv config");
    println!("    invite        Invite a user to the current project <email> [--role <writer|reader>]");
    println!("    invites       Manage invitations: invites list | invites cancel <id> | invites accept <token>");
    println!("    list, ls      List your projects, the current project's environments, or its members");
    println!("                  Usage: list projects | list environments | list members");
    println!("    members       Update member access (owner only)");
    println!("                  Usage: members role <email> <writer|reader> | members environment <email>");
    println!("    delete, d     Delete a project or environment");
    println!("                  Usage: delete project [name] | delete environment <name> [-y|--yes]");
    println!("                  Bare 'delete project' deletes the current project; project deletion requires your password");
    println!("    help, h       Display this help message");
    println!();
    println!("GLOBAL OPTIONS:");
    println!("    --insecure    Allow plaintext HTTP to a non-local host (not recommended)");
    println!("    --profile     Use a named keyring slot for this invocation (e.g. --profile work)");
    println!("    --cacert PATH Add a private CA bundle (PEM) on top of the platform trust store");
    println!();
    println!("OUTPUT:");
    println!("    Variable values are hidden by default. Pass --show-values to reveal");
    println!("    sanitized values when pushing or pulling.");
}
