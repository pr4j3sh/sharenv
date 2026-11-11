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
