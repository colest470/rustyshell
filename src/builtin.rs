use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::fs;

use crate::exec::{find_in_path, write_files};
use crate::helpers::{read_file, remove_trailing_new_line, unquote};

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

pub fn echo(input: &str) {
    let remainder = input.strip_prefix("echo").unwrap_or(input).trim_start();
    let mut args = split_echo_args(remainder);

    if args.is_empty() {
        println!();
        return;
    }

    // Strip a trailing newline from the last arg (from `remove_trailing_new_line`-like behavior)
    if let Some(last) = args.last_mut() {
        if let Some(stripped) = last.strip_suffix('\n') {
            *last = stripped.to_string();
        }
    }

    // Detect redirection: `> file` or `1> file`
    let mut redirect_idx: Option<usize> = None;
    for (i, element) in args.iter().enumerate() {
        if element == ">" || element == "1>" {
            redirect_idx = Some(i);
            break;
        }
    }

    let content_args: Vec<&str> = match redirect_idx {
        Some(i) => args.iter().take(i).filter(|s| !s.is_empty()).map(String::as_str).collect(),
        None => args.iter().filter(|s| !s.is_empty()).map(String::as_str).collect(),
    };

    let content = content_args.join(" ");

    if let Some(i) = redirect_idx {
        if i + 1 < args.len() {
            let file_name = args[i + 1].as_str();
            let content_with_newline = format!("{}\n", content);
            if let Err(e) = write_files(&content_with_newline, file_name) {
                eprintln!("echo: {}: {}", file_name, e);
            }
        } else {
            eprintln!("echo: syntax error: expected filename after '>'");
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

pub fn cat(args: &mut [&str]) {
    let redirect_pos = args.iter().position(|&x| x == "1>" || x == ">" || x == ">>");

    match redirect_pos {
        Some(i) => {
            let paths = &args[..i];
            let dest = args[i + 1..].join(" ");

            let mut content = String::new();
            for path in paths {
                match read_file(path) {
                    Ok(c) => content.push_str(&c),
                    Err(e) => {
                        let msg = e.to_string();
                        let clean = msg.split(" (os error").next().unwrap_or(&msg);
                        eprintln!("cat: {path}: {clean}");
                    }
                }
            }
            if let Some(parent) = std::path::Path::new(&dest).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }

            if let Err(e) = write_files(&content, dest.as_str()) {
                let msg = e.to_string();
                let clean = msg.split(" (os error").next().unwrap_or(&msg);
                eprintln!("cat: {dest}: {clean}");
            }
        }
        None => {
            for path in args.iter() {
                match read_file(path) {
                    Ok(content) => print!("{content}"),
                    Err(e) => {
                        let msg = e.to_string();
                        let clean = msg.split(" (os error").next().unwrap_or(&msg);
                        eprintln!("cat: {path}: {clean}");
                    }
                }
            }
        }
    }
}

pub fn ls(args: &mut [&str]) {
    remove_trailing_new_line(args);

    // 1. Look for redirection operator ('>' or '1>') anywhere in args
    let mut redirect_idx = None;
    for (i, element) in args.iter().enumerate() {
        if *element == ">" || *element == "1>" {
            redirect_idx = Some(i);
            break;
        }
    }

    if let Some(idx) = redirect_idx {
        if idx + 1 < args.len() {
            let file_name = args[idx + 1];
            let ls_args = &args[..idx];

            if let Some(parent) = Path::new(file_name).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = fs::create_dir_all(parent);
                }
            }

            match File::create(file_name) {
                Ok(mut file) => {
                    execute_ls(ls_args, &mut file);
                }
                Err(e) => eprintln!("ls: {}: {}", file_name, e),
            }
            return;
        }
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();
    execute_ls(args, &mut out);
}

fn execute_ls(args: &[&str], out: &mut impl Write) {
    let full_args_str = args.join(" ");
    let unquoted_args = unquote(&full_args_str);

    let mut target_paths: Vec<String> = Vec::new();

    for arg in unquoted_args {
        let trimmed = arg.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('-') {
            continue;
        }
        target_paths.push(trimmed.to_string());
    }

    if target_paths.is_empty() {
        target_paths.push(".".to_string());
    }

    for filename in &target_paths {
        match fs::read_dir(filename) {
            Ok(entries) => {
                let mut names: Vec<String> = Vec::new();
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            names.push(entry.file_name().to_string_lossy().into_owned());
                        }
                        Err(e) => eprintln!("ls: {}: {}", filename, e),
                    }
                }
                names.sort();

                for name in names {
                    if let Err(e) = writeln!(out, "{}", name) {
                        if e.kind() == io::ErrorKind::BrokenPipe {
                            return;
                        }
                    }
                }
            }
            Err(e) => eprintln!("ls: {}: {}", filename, e),
        }
    }

    let _ = out.flush();
}