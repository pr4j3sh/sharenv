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
