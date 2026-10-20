use std::env;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);

    let cmd = args[1].clone();
    println!("{}", cmd);

    if cmd == "init" || cmd == "i" {
        println!("initializing");
        let path = env::current_dir()?;
        let cwd_name = path.file_name().unwrap_or_default().to_string_lossy();
        println!("creating project {}", cwd_name);
    } else if cmd == "add" || cmd == "a" {
        println!("adding")
    }
    Ok(())
}
