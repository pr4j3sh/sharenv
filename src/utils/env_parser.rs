use std::error::Error;

use crate::utils::fs_secure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVariable {
    pub name: String,
    pub value: String,
}

pub fn read_env_file(path: &str) -> Result<Vec<EnvVariable>, Box<dyn Error>> {
    let content = fs_secure::safe_read(std::path::Path::new(path))?;

    parse_env(&content, path)
}

/// Parse a single `.env` line. Returns `None` for blank lines and comments.
pub fn parse_line(raw_line: &str) -> Option<Result<EnvVariable, String>> {
    let line = raw_line.trim();

    if line.is_empty() || line.starts_with('#') {
        return None;
    }

    let line = match line.strip_prefix("export") {
        Some(rest) if rest.starts_with(char::is_whitespace) => rest.trim_start(),
        _ => line,
    };

    let (name, raw_value) = match line.split_once('=') {
        Some(parts) => parts,
        None => return Some(Err("expected KEY=VALUE".to_string())),
    };

    let name = name.trim();

    if name.is_empty() {
        return Some(Err("empty variable name".to_string()));
    }

    Some(Ok(EnvVariable {
        name: name.to_string(),
        value: parse_value(raw_value.trim()),
    }))
}

/// Format a variable as a `.env` line, quoting values when needed.
pub fn format_line(variable: &EnvVariable) -> String {
    format!("{}={}", variable.name, format_value(&variable.value))
}

fn parse_env(content: &str, path: &str) -> Result<Vec<EnvVariable>, Box<dyn Error>> {
    let mut variables: Vec<EnvVariable> = Vec::new();

    for (index, raw_line) in content.lines().enumerate() {
        let line_number = index + 1;

        let variable = match parse_line(raw_line) {
            None => continue,
            Some(Ok(variable)) => variable,
            Some(Err(error)) => {
                return Err(
                    format!("invalid entry in {path} at line {line_number}: {error}").into(),
                );
            }
        };

        if variables
            .iter()
            .any(|existing| existing.name == variable.name)
        {
            return Err(format!(
                "duplicate variable '{}' in {path} at line {line_number}",
                variable.name
            )
            .into());
        }

        variables.push(variable);
    }

    Ok(variables)
}

fn parse_value(raw: &str) -> String {
    if raw.len() >= 2 {
        let bytes = raw.as_bytes();
        let first = bytes[0];
        let last = bytes[raw.len() - 1];

        if first == b'"' && last == b'"' {
            return unescape_double_quoted(&raw[1..raw.len() - 1]);
        }

        if first == b'\'' && last == b'\'' {
            return raw[1..raw.len() - 1].to_string();
        }
    }

    strip_inline_comment(raw)
}

fn unescape_double_quoted(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars();

    while let Some(character) = chars.next() {
        if character != '\\' {
            result.push(character);
            continue;
        }

        match chars.next() {
            Some('n') => result.push('\n'),
            Some('r') => result.push('\r'),
            Some('t') => result.push('\t'),
            Some('\\') => result.push('\\'),
            Some('"') => result.push('"'),
            Some(other) => {
                result.push('\\');
                result.push(other);
            }
            None => result.push('\\'),
        }
    }

    result
}

fn strip_inline_comment(raw: &str) -> String {
    let bytes = raw.as_bytes();

    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'#' && index > 0 && matches!(bytes[index - 1], b' ' | b'\t') {
            return raw[..index].trim_end().to_string();
        }
    }

    raw.to_string()
}

fn format_value(value: &str) -> String {
    let needs_quotes = value.is_empty()
        || value.chars().any(|character| {
            character.is_whitespace() || matches!(character, '#' | '"' | '\'' | '\\')
        });

    if !needs_quotes {
        return value.to_string();
    }

    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");

    format!("\"{escaped}\"")
}
