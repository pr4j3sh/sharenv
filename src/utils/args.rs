/// Remove boolean switches from `args` and report whether any were present.
pub fn parse_switch(args: &[String], names: &[&str]) -> (Vec<String>, bool) {
    let mut remaining = Vec::with_capacity(args.len());
    let mut found = false;

    for arg in args {
        if names.contains(&arg.as_str()) {
            found = true;
        } else {
            remaining.push(arg.clone());
        }
    }

    (remaining, found)
}

/// Remove `--profile <name>` from `args` and return it validated+normalized.
/// Profile namespaces the keyring slot for this invocation only; it cannot be
/// persisted in config or environment variables.
pub fn take_profile(args: &[String]) -> Result<(Vec<String>, Option<String>), Box<dyn std::error::Error>> {
    let mut remaining = Vec::with_capacity(args.len());
    let mut profile: Option<String> = None;
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--profile" => {
                let value = iter.next().ok_or("missing value for --profile")?;

                if value.starts_with('-') {
                    return Err("missing value for --profile".into());
                }

                profile = Some(crate::utils::token::validate_profile(value)?);
            }
            value if value.starts_with("--profile=") => {
                let value = &value["--profile=".len()..];

                if value.is_empty() {
                    return Err("missing value for --profile".into());
                }

                profile = Some(crate::utils::token::validate_profile(value)?);
            }
            _ => remaining.push(arg.clone()),
        }
    }

    Ok((remaining, profile))
}

/// Take at most one positional argument from `args`; unknown flags and extra
/// positionals are rejected.
pub fn take_positional(args: &[String]) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut found: Option<String> = None;

    for arg in args {
        if arg.starts_with('-') {
            return Err(format!("unexpected argument '{arg}'").into());
        }

        if found.is_some() {
            return Err(format!("unexpected argument '{arg}'").into());
        }

        found = Some(arg.clone());
    }

    Ok(found)
}
