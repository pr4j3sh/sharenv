use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

pub fn create_env_file(path: &str) -> io::Result<()> {
    if Path::new(path).exists() {
        print!("{path} already exists. Replace it? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().eq_ignore_ascii_case("y") {
            File::create(path)?;
            println!("Replaced {path}");
        } else {
            println!("Keeping existing {path}");
        }
    } else {
        File::create(path)?;
        println!("Created {path}");
    }

    Ok(())
}

pub fn get_env_name(file: &str) -> Result<String, Box<dyn std::error::Error>> {
    let name = std::path::Path::new(file)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Invalid environment filename")?;

    let env_name = name
        .strip_prefix(".env.")
        .ok_or(format!("Invalid environment filename: {file}"))?;

    Ok(env_name.to_string())
}
