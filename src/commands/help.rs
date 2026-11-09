use crate::settings::AppConfig;

pub fn run(config: &AppConfig) {
    let name = env!("CARGO_PKG_NAME");

    println!("{name} - Manage your project environments\n");
    println!("USAGE:");
    println!("    {name} <COMMAND>\n");
    println!("    using config v{} \n", &config.config_version);
    println!("COMMANDS:");
    println!("    register, r   Register a new account");
    println!("    login, l      User login");
    println!("    init, i       Initialize a project [-p|--project <project_name>]");
    println!("    add, a        Share the current project with a user [-u|--user <email>]");
    println!("                  Create an environment [-e|--environment <env_name>]");
    println!("    push          Push variables [-e|--environment <env_name>]");
    println!("                  Omit -e to push all environments with local .env files");
    println!("    pull          Pull variables [-e|--environment <env_name>]");
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
    println!("    help, h       Display this help message");
}
