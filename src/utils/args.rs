pub fn parse_value_arg(
    args: &[String],
    long: &str,
    short: &str,
    label: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let long_eq = format!("{long}=");
    let short_eq = format!("{short}=");
    let mut parsed = None;
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        let value = match arg.as_str() {
            value if value == long || value == short => {
                let value = iter
                    .next()
                    .ok_or_else(|| format!("missing value for {arg}"))?;

                if value.starts_with('-') {
                    return Err(format!("missing value for {arg}").into());
                }

                value
            }
            _ => {
                if let Some(value) = arg.strip_prefix(long_eq.as_str()) {
                    value
                } else if let Some(value) = arg.strip_prefix(short_eq.as_str()) {
                    value
                } else {
                    return Err(format!("unexpected argument '{arg}'").into());
                }
            }
        };

        if value.is_empty() {
            return Err(format!("{label} cannot be empty").into());
        }

        parsed = Some(value.to_string());
    }

    Ok(parsed)
}

pub fn parse_name_arg(args: &[String]) -> Result<Option<String>, Box<dyn std::error::Error>> {
    parse_value_arg(args, "--name", "-n", "project name")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn parses_long_and_short_flags() {
        assert_eq!(
            parse_value_arg(
                &args(&["--environment", "dev"]),
                "--environment",
                "-e",
                "name"
            )
            .unwrap(),
            Some("dev".to_string())
        );
        assert_eq!(
            parse_value_arg(&args(&["-e", "prod"]), "--environment", "-e", "name").unwrap(),
            Some("prod".to_string())
        );
    }

    #[test]
    fn parses_equals_form() {
        assert_eq!(
            parse_value_arg(&args(&["--environment=dev"]), "--environment", "-e", "name").unwrap(),
            Some("dev".to_string())
        );
        assert_eq!(
            parse_value_arg(&args(&["-e=dev"]), "--environment", "-e", "name").unwrap(),
            Some("dev".to_string())
        );
    }

    #[test]
    fn rejects_missing_and_empty_values() {
        assert!(parse_value_arg(&args(&["--environment"]), "--environment", "-e", "name").is_err());
        assert!(
            parse_value_arg(&args(&["--environment="]), "--environment", "-e", "name").is_err()
        );
    }

    #[test]
    fn rejects_unexpected_arguments() {
        assert!(parse_value_arg(&args(&["dev"]), "--environment", "-e", "name").is_err());
    }

    #[test]
    fn parses_name_argument() {
        assert_eq!(
            parse_name_arg(&args(&["--name", "my-project"])).unwrap(),
            Some("my-project".to_string())
        );
    }
}
