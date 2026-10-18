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

pub fn echo(args: &mut [&str]) {
    if args.is_empty() {
        println!();
        return;
    }

    // Strip a trailing newline from the last arg (from `remove_trailing_new_line`-like behavior)
    let last_idx = args.len() - 1;
    if let Some(stripped) = args[last_idx].strip_suffix('\n') {
        args[last_idx] = stripped;
    }

    // Detect redirection: `> file` or `1> file`
    let mut redirect_idx: Option<usize> = None;
    for (i, element) in args.iter().enumerate() {
        if *element == ">" || *element == "1>" {
            redirect_idx = Some(i);
            break;
        }
    }

    let content_args: Vec<&str> = match redirect_idx {
        Some(i) => args[..i].iter().copied().filter(|s| !s.is_empty()).collect(),
        None => args.iter().copied().filter(|s| !s.is_empty()).collect(),
    };

    let content = parse_echo_args(&content_args);

    if let Some(i) = redirect_idx {
        if i + 1 < args.len() {
            let file_name = args[i + 1];
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

fn parse_echo_args(args: &[&str]) -> String {
    let input = args.join(" ");
    let mut result = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut last_was_space = false;
    let mut chars = input.chars().peekable();

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

    result
}

pub fn cat(args: &mut [&str]) {
    let redirect_pos = args.iter().position(|&x| x == "1>" || x == ">" || x == ">>");

    match redirect_pos {
        Some(i) => {
            let paths = &args[..i];
            let dest = args[i + 1];

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

            if let Some(parent) = std::path::Path::new(dest).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }

            if let Err(e) = write_files(&content, dest) {
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