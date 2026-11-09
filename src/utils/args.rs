use std::collections::HashMap;

pub struct FlagSpec<'a> {
    pub key: &'static str,
    pub names: &'a [&'a str],
    pub label: &'a str,
}

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

pub fn parse_project_arg(args: &[String]) -> Result<Option<String>, Box<dyn std::error::Error>> {
    parse_value_arg(args, "--project", "-p", "project name")
}

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
