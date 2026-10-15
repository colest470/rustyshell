use std::fs::File;
use std::io::{self, Write};

use crate::exec::{find_in_path, write_files};
use crate::helpers::{remove_trailing_new_line, unquote};

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
    let args_length = args.len();
    if args_length == 0 {
        println!();
        return;
    }

    let mut last_element = args[args_length - 1];
    if let Some(stripped) = last_element.strip_suffix('\n') {
        last_element = stripped;
    }
    args[args_length - 1] = last_element;

    let input = args.join(" ");
    let mut result = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut last_was_space = false;
    let mut chars = input.chars().peekable();

    // implementation for redirect output
    let file_checker: Vec<&str> = ["1>", ">"].to_vec();

    for (i, element) in args.iter().enumerate() {
        if *element == file_checker[0] || *element == file_checker[1] {
            let file_name = args[i + 1];
            let content = args[0..i].join(" ");

            let result = write_files(content.as_str(), file_name); // result : io::Result<()>
            match result {
                Ok(_) => {
                    println!("Content written to file: {}", file_name);
                }
                Err(e) => {
                    eprintln!("Error writing to file {}: {}", file_name, e);
                }
            }
            continue;
        }
    }

    while let Some(ch) = chars.next() {
        match ch {
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
                last_was_space = false;
            }
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
                last_was_space = false;
            }
            '\\' if !in_single_quotes => {
                match chars.next() {
                    Some(next_ch) => {
                        if in_double_quotes
                            && !matches!(next_ch, '"' | '\\' | '$' | '`' | '\n')
                        {
                            result.push('\\');
                            result.push(next_ch);
                        } else {
                            result.push(next_ch);
                        }
                    }
                    None => result.push('\\'),
                }
                last_was_space = false;
            }
            ' ' | '\t' if !in_single_quotes && !in_double_quotes => {
                if !last_was_space && !result.is_empty() {
                    result.push(' ');
                }
                last_was_space = true;
            }
            _ => {
                result.push(ch);
                last_was_space = false;
            }
        }
    }

    println!("{}", result);
}

pub fn cat(args: &mut [&str]) {
    remove_trailing_new_line(args);

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

pub fn ls(args: &mut [&str]) {
    remove_trailing_new_line(args);

    let full_args = args.join(" ");
    let unquoted_args = unquote(&full_args);

    let stdout = io::stdout();
    let mut out = stdout.lock();

    for element in args.iter() {
        if *element == ">" || *element == "1>" {
            let file_name = args[args.len() - 1];
            let content = args[0..args.len() - 2].join(" ");

            let result = write_files(content.as_str(), file_name); // result : io::Result<()>
            match result {
                Ok(_) => {
                    print!("$ ");
                }
                Err(e) => {
                    eprintln!("Error writing to file {}: {}", file_name, e);
                }
            }
            return;
        }
    }

    for filename in unquoted_args {
        let filename = filename.trim();
        if filename.is_empty() {
            continue;
        }

        match std::fs::read_dir(filename) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            if let Err(e) = writeln!(out, "{}", entry.file_name().to_string_lossy()) {
                                if e.kind() == io::ErrorKind::BrokenPipe {
                                    return;
                                }
                                eprintln!("ls: {}: {}", filename, e);
                            }
                        }
                        Err(e) => eprintln!("ls: {}: {}", filename, e),
                    }
                }
            }
            Err(e) => eprintln!("ls: {}: {}", filename, e),
        }
    }

    let _ = out.flush();
}