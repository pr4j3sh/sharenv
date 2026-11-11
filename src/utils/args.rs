/// Remove boolean switches from `args` and report whether any were present.
pub fn parse_switch(args: &[String], names: &[&str]) -> (Vec<String>, bool) {
    let (remaining, found, _) = parse_switches(args, names, &[]);

    (remaining, found)
}

/// Remove two groups of boolean switches from `args` and report which were
/// present.
pub fn parse_switches(
    args: &[String],
    first: &[&str],
    second: &[&str],
) -> (Vec<String>, bool, bool) {
    let mut remaining = Vec::with_capacity(args.len());
    let mut first_found = false;
    let mut second_found = false;

    for arg in args {
        if first.contains(&arg.as_str()) {
            first_found = true;
        } else if second.contains(&arg.as_str()) {
            second_found = true;
        } else {
            remaining.push(arg.clone());
        }
    }

    (remaining, first_found, second_found)
}

/// Remove `--cacert <path>` from `args` and return the path. Additive to the
/// platform trust store; per-invocation only, never persisted.
pub fn take_cacert(
    args: &[String],
) -> Result<(Vec<String>, Option<String>), Box<dyn std::error::Error>> {
    let mut remaining = Vec::with_capacity(args.len());
    let mut cacert: Option<String> = None;
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--cacert" => {
                let value = iter.next().ok_or("missing value for --cacert")?;

                if value.starts_with('-') {
                    return Err("missing value for --cacert".into());
                }

                let path = std::path::Path::new(value);

                if !path.is_file() {
                    return Err(format!("--cacert '{value}' is not a file").into());
                }

                cacert = Some(value.clone());
            }
            value if value.starts_with("--cacert=") => {
                let value = &value["--cacert=".len()..];

                if value.is_empty() {
                    return Err("missing value for --cacert".into());
                }

                let path = std::path::Path::new(value);

                if !path.is_file() {
                    return Err(format!("--cacert '{value}' is not a file").into());
                }

                cacert = Some(value.to_string());
            }
            _ => remaining.push(arg.clone()),
        }
    }

    Ok((remaining, cacert))
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
