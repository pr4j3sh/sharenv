pub fn parse_name_arg(args: &[String]) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut name = None;
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        let value = match arg.as_str() {
            "--name" | "-n" => {
                let value = iter
                    .next()
                    .ok_or_else(|| format!("missing value for {arg}"))?;

                if value.starts_with('-') {
                    return Err(format!("missing value for {arg}").into());
                }

                value
            }
            _ => {
                if let Some(value) = arg.strip_prefix("--name=") {
                    value
                } else if let Some(value) = arg.strip_prefix("-n=") {
                    value
                } else {
                    return Err(format!("unexpected argument '{arg}'").into());
                }
            }
        };

        if value.is_empty() {
            return Err("project name cannot be empty".into());
        }

        name = Some(value.to_string());
    }

    Ok(name)
}
