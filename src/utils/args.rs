use std::collections::HashMap;

/// One flag accepted by [`parse_flags`], along with every name it answers to.
pub struct FlagSpec<'a> {
    pub key: &'static str,
    pub names: &'a [&'a str],
    pub label: &'a str,
}

/// Parse every argument in one pass, allowing multiple flags and aliases.
///
/// Both `--flag value` and `--flag=value` forms are accepted. Arguments that do
/// not match a known flag are rejected.
pub fn parse_flags<'a>(
    args: &[String],
    specs: &[FlagSpec<'a>],
) -> Result<HashMap<&'static str, String>, Box<dyn std::error::Error>> {
    let mut values = HashMap::new();
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        if let Some(spec) = specs
            .iter()
            .find(|spec| spec.names.iter().any(|name| *name == arg.as_str()))
        {
            let value = iter
                .next()
                .ok_or_else(|| format!("missing value for {arg}"))?;

            if value.starts_with('-') {
                return Err(format!("missing value for {arg}").into());
            }

            insert_value(&mut values, spec, value)?;
            continue;
        }

        let matched = specs.iter().find_map(|spec| {
            spec.names.iter().find_map(|name| {
                arg.strip_prefix(*name)
                    .and_then(|rest| rest.strip_prefix('='))
                    .map(|value| (spec, value))
            })
        });

        match matched {
            Some((spec, value)) => insert_value(&mut values, spec, value)?,
            None => return Err(format!("unexpected argument '{arg}'").into()),
        }
    }

    Ok(values)
}

fn insert_value(
    values: &mut HashMap<&'static str, String>,
    spec: &FlagSpec<'_>,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if value.is_empty() {
        return Err(format!("{} cannot be empty", spec.label).into());
    }

    values.insert(spec.key, value.to_string());

    Ok(())
}

pub fn parse_value_arg(
    args: &[String],
    long: &str,
    short: &str,
    label: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let names = [long, short];
    let specs = [FlagSpec {
        key: "value",
        names: &names,
        label,
    }];

    Ok(parse_flags(args, &specs)?.remove("value"))
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

    fn pull_specs() -> [FlagSpec<'static>; 2] {
        [
            FlagSpec {
                key: "environment",
                names: &["--environment", "-e"],
                label: "environment name",
            },
            FlagSpec {
                key: "project_id",
                names: &["--pid", "--project_id", "-pid"],
                label: "project id",
            },
        ]
    }

    #[test]
    fn parses_multiple_flags_in_one_pass() {
        let values = parse_flags(
            &args(&["--pid", "6aeb272225e009da54a9dbdb", "-e", "dev"]),
            &pull_specs(),
        )
        .unwrap();

        assert_eq!(
            values.get("project_id").map(String::as_str),
            Some("6aeb272225e009da54a9dbdb")
        );
        assert_eq!(values.get("environment").map(String::as_str), Some("dev"));
    }

    #[test]
    fn parses_project_id_aliases() {
        for flag in ["--pid=abc", "--project_id=abc", "-pid=abc"] {
            let values = parse_flags(&args(&[flag, "-e", "dev"]), &pull_specs()).unwrap();
            assert_eq!(
                values.get("project_id").map(String::as_str),
                Some("abc"),
                "failed for {flag}"
            );
        }

        for flag in ["--pid", "--project_id", "-pid"] {
            let values = parse_flags(&args(&[flag, "abc", "-e", "dev"]), &pull_specs()).unwrap();
            assert_eq!(
                values.get("project_id").map(String::as_str),
                Some("abc"),
                "failed for {flag}"
            );
        }
    }

    #[test]
    fn rejects_unknown_flags_alongside_known_ones() {
        let error = parse_flags(&args(&["-e", "dev", "--bogus"]), &pull_specs()).unwrap_err();

        assert_eq!(error.to_string(), "unexpected argument '--bogus'");
    }

    #[test]
    fn rejects_missing_project_id_value() {
        let error = parse_flags(&args(&["-e", "dev", "--pid"]), &pull_specs()).unwrap_err();

        assert_eq!(error.to_string(), "missing value for --pid");
    }
}
