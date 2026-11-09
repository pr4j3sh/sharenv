//! Terminal-safe formatting for untrusted strings.
//!
//! Variable values, names, and API error text are attacker-influenced. Strip
//! ANSI escape sequences and render control characters visibly before writing
//! them to a terminal.

use std::iter::Peekable;
use std::str::Chars;

const MAX_CHARS: usize = 200;

pub fn sanitize(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(character) = chars.next() {
        match character {
            '\u{1b}' => skip_escape(&mut chars),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            other if other.is_control() => {
                use std::fmt::Write;
                let _ = write!(output, "\\u{{{:x}}}", other as u32);
            }
            other => output.push(other),
        }
    }

    truncate(&output)
}

fn skip_escape(chars: &mut Peekable<Chars<'_>>) {
    match chars.peek().copied() {
        // CSI: ESC [ ... final byte in the 0x40-0x7e range.
        Some('[') => {
            chars.next();
            for character in chars.by_ref() {
                if ('@'..='~').contains(&character) {
                    break;
                }
            }
        }
        // OSC: ESC ] ... terminated by BEL or ESC \.
        Some(']') => {
            chars.next();
            while let Some(character) = chars.next() {
                if character == '\u{7}' {
                    break;
                }
                if character == '\u{1b}' {
                    if chars.peek() == Some(&'\\') {
                        chars.next();
                    }
                    break;
                }
            }
        }
        _ => {}
    }
}

fn truncate(input: &str) -> String {
    if input.chars().count() <= MAX_CHARS {
        return input.to_string();
    }

    let mut output = input.chars().take(MAX_CHARS - 1).collect::<String>();
    output.push('…');
    output
}

#[cfg(test)]
mod tests {
    use super::sanitize;

    #[test]
    fn strips_ansi_escapes() {
        assert_eq!(sanitize("\u{1b}[31mred\u{1b}[0m"), "red");
        assert_eq!(sanitize("\u{1b}]0;title\u{7}text"), "text");
    }

    #[test]
    fn renders_control_characters() {
        assert_eq!(sanitize("a\nb\r\tc"), "a\\nb\\r\\tc");
        assert_eq!(sanitize("a\u{0}b"), "a\\u{0}b");
    }

    #[test]
    fn truncates_long_values() {
        let value = "a".repeat(500);
        let output = sanitize(&value);
        assert_eq!(output.chars().count(), 200);
        assert!(output.ends_with('…'));
    }
}
