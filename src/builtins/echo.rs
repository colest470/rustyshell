use crate::exec::{append_to_file, write_files};

use crate::exec::{append_to_file, write_files};

pub fn echo(input: &str) {
    let remainder = input.strip_prefix("echo").unwrap_or(input).trim_start();
    let args = split_echo_args(remainder);

    let stdout_idx = args.iter().position(|a| a == ">" || a == "1>");
    let stderr_idx = args.iter().position(|a| a == "2>");
    let append_idx = args.iter().position(|a| a == ">>" || a == "1>>");
    let stderr_append_idx = args.iter().position(|a| a == "2>>");

    let first_redirect = [stdout_idx, stderr_idx, append_idx, stderr_append_idx]
        .into_iter()
        .flatten()
        .min();
    let content_end = first_redirect.unwrap_or(args.len());
    let content = args[..content_end].join(" ");

    // stderr redirects: echo never writes to stderr, but the shell still
    // creates/truncates (2>) or creates (2>>) the target file.
    if let Some(i) = stderr_idx {
        match args.get(i + 1) {
            Some(fname) => {
                if let Err(e) = write_files("", fname) {
                    eprintln!("echo: {}: {}", fname, e);
                }
            }
            None => eprintln!("echo: syntax error: expected filename after '2>'"),
        }
    }

    if let Some(i) = stderr_append_idx {
        match args.get(i + 1) {
            Some(fname) => {
                if let Err(e) = append_to_file("", fname) {
                    eprintln!("echo: {}: {}", fname, e);
                }
            }
            None => eprintln!("echo: syntax error: expected filename after '2>>'"),
        }
    }

    if let Some(i) = append_idx {
        match args.get(i + 1) {
            Some(fname) => {
                if let Err(e) = append_to_file(&format!("{}\n", content), fname) {
                    eprintln!("echo: {}: {}", fname, e);
                }
            }
            None => eprintln!("echo: syntax error: expected filename after '>>'"),
        }
        return;
    }

    if let Some(i) = stdout_idx {
        match args.get(i + 1) {
            Some(fname) => {
                if let Err(e) = write_files(&format!("{}\n", content), fname) {
                    eprintln!("echo: {}: {}", fname, e);
                }
            }
            None => eprintln!("echo: syntax error: expected filename after '>'"),
        }
        return;
    }

    println!("{}", content);
}

fn split_echo_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            '\\' if !in_single => {
                match chars.next() {
                    Some(next_ch) => {
                        if in_double && !matches!(next_ch, '"' | '\\' | '$' | '`' | '\n') {
                            current.push('\\');
                        }
                        current.push(next_ch);
                    }
                    None => current.push('\\'),
                }
            }
            ' ' | '\t' if !in_single && !in_double => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}