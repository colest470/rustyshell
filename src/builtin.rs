use std::fs::File;
use std::io::{self, Write};

use crate::exec::{find_in_path};
use crate::helpers::{remove_trailing_new_line, unquote}; // this has an error

pub fn builtins (args: &[&str]) {
    let target = args.first().copied().unwrap_or("");

    if matches!(target, "type" | "exit" | "echo") {
        println!("{} is a shell builtin", target);
    } else if let Some(path) = find_in_path(target) {
        println!("{} is {}", target, path.display());
    } else {
        println!("{}: not found", target);
    }
}

pub fn echo(args: &mut [&str]) {
    // Remove the trailing newline from the last element
    let args_length = args.len();
    if args_length == 0 {
        println!();
        return;
    }
    
    let mut last_element = args[args_length - 1];
    if last_element.ends_with('\n') {
        last_element = &last_element[..last_element.len() - 1];
    }
    args[args_length - 1] = last_element;

    let input = args.join(" ");
    let mut result = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut last_was_space = false;

    for ch in input.chars() {
        match ch {
            '\'' if !in_double_quotes => in_single_quotes = !in_single_quotes,
            '"' if !in_single_quotes => in_double_quotes = !in_double_quotes,
            ' ' if !in_single_quotes && !in_double_quotes => {
                if !last_was_space && !result.is_empty() {
                    result.push(' ');
                }
                last_was_space = true;
                continue;
            }
            _ => result.push(ch),
        }
        last_was_space = false;
    }

    println!("{}", result);
}

pub fn cat(mut args: &mut [&str]) {
    args = remove_trailing_new_line(args);

    let full_args = args.join(" ");
    let unquoted_args = unquote(&full_args);

    let stdout = io::stdout();
    let mut out = stdout.lock();

    for filename in unquoted_args {
        let filename = filename.trim();
        if filename.is_empty() {
            continue;
        }

        match File::open(filename) {
            Ok(mut file) => {
                if let Err(e) = io::copy(&mut file, &mut out) {
                    if e.kind() == io::ErrorKind::BrokenPipe {
                        return;
                    }
                    eprintln!("cat: {}: {}", filename, e);
                }
            }
            Err(e) => eprintln!("cat: {}: {}", filename, e),
        }
    }

    let _ = out.flush();
}