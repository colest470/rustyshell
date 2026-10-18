use std::fs;
use std::io;

use crate::exec::{find_in_path};

pub fn remove_trailing_new_line<'a, 'b>(args: &'a mut [&'b str]) -> &'a mut [&'b str] {
    if let Some(last) = args.last_mut() {
        if last.ends_with('\n') {
            *last = &last[..last.len() - 1];
        }
        if last.ends_with('\r') {
            *last = &last[..last.len() - 1];
        }
    }
    args
}


pub fn unquote(arg: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut has_content = false;
    let mut chars = arg.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\'' if !in_double => {
                in_single = !in_single;
                has_content = true;
            }
            '"' if !in_single => {
                in_double = !in_double;
                has_content = true;
            }
            '\\' if !in_single => {
                if let Some(next) = chars.next() {
                    if in_double && !matches!(next, '"' | '\\' | '$' | '`' | '\n') {
                        current.push('\\');
                        current.push(next);
                    } else {
                        current.push(next);
                    }
                    has_content = true;
                } else {
                    current.push('\\');
                    has_content = true;
                }
            }
            ' ' | '\t' if !in_single && !in_double => {
                if has_content {
                    result.push(std::mem::take(&mut current));
                    has_content = false;
                }
            }
            _ => {
                current.push(ch);
                has_content = true;
            }
        }
    }

    if has_content {
        result.push(current);
    }

    result
}

pub fn if_executable(input: &str) -> (bool, String, String, Vec<String>) {
    let parts = unquote(input);

    let opt = find_in_path(parts[0].as_str());

    if let Some(path) = opt {
        let command = parts[0].clone();
        let path_to_file = path.to_str().unwrap().to_string();
        let args: Vec<String> = parts[1..].to_vec();

        return (true, command, path_to_file, args);
    }

    (false, String::new(), String::new(), Vec::new())
}

pub fn read_file(path: &str) -> io::Result<String> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}

pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    let mut token_started = false;

    while let Some(c) = chars.next() {
        match (in_single, in_double, c) {
            (true, _, '\'') => { in_single = false; token_started = true; }
            (_, true, '"') => { in_double = false; token_started = true; }
            (false, false, '\'') => { in_single = true; token_started = true; }
            (false, false, '"') => { in_double = true; token_started = true; }
            (true, _, _) => current.push(c),      // literal in single quotes
            (_, true, '\\') => {
                match chars.next() {
                    Some('"')  => current.push('"'),
                    Some('\\') => current.push('\\'),
                    Some('$')  => current.push('$'),
                    Some(other) => { current.push('\\'); current.push(other); }
                    None => current.push('\\'),
                }
            }
            (_, true, _) => current.push(c),
            (false, false, '\\') => {
                if let Some(next) = chars.next() { current.push(next); }
            }
            (false, false, ' ' | '\t') if token_started => {
                tokens.push(std::mem::take(&mut current));
                token_started = false;
            }
            (false, false, ' ' | '\t') => { /* skip extra whitespace */ }
            (false, false, _) => { current.push(c); token_started = true; }
        }
    }
    if token_started || !current.is_empty() {
        tokens.push(current);
    }
    tokens
}