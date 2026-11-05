pub fn run() {
    let name = env!("CARGO_PKG_NAME");

    println!("{name} - Manage your project environments\n");
    println!("USAGE:");
    println!("    {name} <COMMAND>\n");
    println!("COMMANDS:");
    println!("    register, r   Register a new account");
    println!("    login, l      User login");
    println!("    init, i       Initialize a project [-n|--name <project_name>]");
    println!("    add, a        Share the current project with a user [-u|--user <email>]");
    println!("    push          Push variables for an environment [-e|--environment <env_name>]");
    println!("    pull          Pull variables for an environment [-e|--environment <env_name>]");
    println!("                  Use [-pid|--project_id <project_id>] to pull a shared project");
    println!("    help, h       Display this help message");
}
