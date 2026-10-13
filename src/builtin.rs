use crate::exec::{find_in_path};
use std::fs::File;
use std::io::{self, Read, Write};

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
    let mut last_element = args[args_length - 1];
    last_element = &last_element[..last_element.len() - 1];
    args[args_length - 1] = last_element;

    let input = args.join(" ");
    
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    let mut in_quotes = false;
    let mut last_was_space = false;
    
    while let Some(ch) = chars.next() {
        match ch {
            '\'' => {
                in_quotes = !in_quotes;
            }
            ' ' => {
                if in_quotes {
                    result.push(ch);
                    last_was_space = true;
                } else {
                    if !last_was_space && !result.is_empty() {
                        result.push(' ');
                        last_was_space = true;
                    }
                }
            }
            _ => {
                result.push(ch);
                last_was_space = false;
            }
        }
    }
    
    println!("{}", result);
}

use std::fs::File;
use std::io::{self, Read, Write};

pub fn cat<'a>(mut args: &'a mut [&'a str]) {
    args = remove_trailing_new_line(args);

    let full_args = args.join(" ");
    let unquoted_args = unquote(&full_args);

    let stdout = io::stdout();
    let mut out = stdout.lock();

    for argument in unquoted_args {
        let filename = argument.trim();
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

pub fn remove_trailing_new_line<'a>(args: &'a mut [&'a str])-> &'a mut [&'a str] {
    let args_length: usize = args.len();
    let mut last_element: &str = args[args_length - 1];
    last_element = &last_element[..last_element.len() - 1];
    args[args_length - 1] = last_element;

    return args;
}


pub fn unquote<'a>(arg: &'a str) -> Vec<&'a str> {
    let mut result: Vec<&'a str> = Vec::new();
    let mut in_quotes = false;
    let mut segment_start = 0usize;

    for (i, ch) in arg.char_indices() {
        match ch {
            '\'' => {
                let segment = &arg[segment_start..i];
                if !segment.is_empty() {
                    result.push(segment);
                }
                in_quotes = !in_quotes;
                segment_start = i + 1;
            }
            _ => {}
        }
    }

    if segment_start < arg.len() {
        let segment = &arg[segment_start..];
        if !segment.is_empty() {
            result.push(segment);
        }
    }

    result
}