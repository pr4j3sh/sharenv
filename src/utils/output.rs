//! Terminal-safe formatting for untrusted strings.
//!
//! Variable values, names, and API error text are attacker-influenced. Strip
//! ANSI escape sequences and render control characters visibly before writing
//! them to a terminal.

use std::iter::Peekable;
use std::str::Chars;
use std::sync::atomic::{AtomicBool, Ordering};

static JSON_OUTPUT: AtomicBool = AtomicBool::new(false);

/// Set once at startup; prompts write to stderr so stdout stays pure JSON.
pub fn set_json_output(value: bool) {
    JSON_OUTPUT.store(value, Ordering::Relaxed);
}

pub fn json_output() -> bool {
    JSON_OUTPUT.load(Ordering::Relaxed)
}

/// Print a progress line. Human output goes to stdout; in `--json` mode it
/// goes to stderr so stdout carries exactly one JSON document.
pub fn line(json: bool, text: impl AsRef<str>) {
    if json {
        eprintln!("{}", text.as_ref());
    } else {
        println!("{}", text.as_ref());
    }
}

/// Print a single JSON document on stdout.
pub fn json<T: serde::Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(text) => println!("{text}"),
        Err(_) => println!("{{}}"),
    }
}

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
