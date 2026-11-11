use std::{
    error::Error,
    io::{self, Write},
};

/// Prompt for a single line of non-secret text.
pub fn prompt_line(message: &str) -> io::Result<String> {
    print!("{message}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim_end_matches(['\r', '\n']).trim().to_string())
}

pub fn confirm(message: &str) -> io::Result<bool> {
    print!("{message}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(matches!(
        input.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

pub fn prompt_credentials() -> Result<(String, String), Box<dyn Error>> {
    print!("email: ");
    io::stdout().flush()?;

    let mut email = String::new();
    io::stdin().read_line(&mut email)?;
    let email = email.trim();

    if email.is_empty() {
        return Err("Email cannot be empty".into());
    }

    let password = rpassword::prompt_password("password: ")?;

    if password.is_empty() {
        return Err("Password cannot be empty".into());
    }

    Ok((email.to_string(), password))
}
