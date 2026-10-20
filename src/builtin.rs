use std::fs::File;
use std::io::{self, Read, Write};
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

    if let Some(last) = args.last_mut() {
        if let Some(stripped) = last.strip_suffix('\n') {
            *last = stripped.to_string();
        }
    }

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
    remove_trailing_new_line(args);

    let rest = &args[1..];

    // Nothing to read from → echo stdin to stdout (like real cat)
    if rest.is_empty() {
        let stdin = io::stdin();
        let mut lock = stdin.lock();
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let mut buf = [0u8; 8192];
        loop {
            match lock.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => { let _ = out.write_all(&buf[..n]); }
                Err(_) => break,
            }
        }
        let _ = out.flush();
        return;
    }

    // Collect path arguments, stopping at a redirection operator.
    // (If the dispatcher already stripped redirections, this loop just
    // copies every element — harmless.)
    let mut paths: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        match rest[i] {
            ">" | "1>" | "2>" => {
                // skip operator + its target path
                i += 2;
                break;
            }
            other => {
                paths.push(other);
                i += 1;
            }
        }
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();

    for path in paths {
        match File::open(path) {
            Ok(mut f) => {
                let mut buf = [0u8; 8192];
                loop {
                    match f.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if out.write_all(&buf[..n]).is_err() {
                                return;
                            }
                        }
                        Err(e) => {
                            eprintln!("cat: {}: {}", path, clean_io(&e));
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                // eprintln → stderr, so `cat missing 1> file` puts the
                // error on the terminal and the file stays empty.
                eprintln!("cat: {}: {}", path, clean_io(&e));
            }
        }
    }
    let _ = out.flush();
}

/// Strip the ` (os error N)` suffix so messages match real `cat`.
fn clean_io(e: &io::Error) -> String {
    let s = e.to_string();
    match s.find(" (os error ") {
        Some(i) => s[..i].to_string(),
        None => s,
    }
}

pub fn ls(args: &mut [&str]) {
    remove_trailing_new_line(args);

    let mut redirect_idx = None;
    let mut is_stderr = false;

    for (i, element) in args.iter().enumerate() {
        if *element == ">" || *element == "1>" {
            redirect_idx = Some(i);
            is_stderr = false;
            break;
        } else if *element == "2>" {
            redirect_idx = Some(i);
            is_stderr = true;
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
                let stdout = io::stdout();
                let mut out_lock = stdout.lock();
                let stderr = io::stderr();
                let mut err_lock = stderr.lock();

                if is_stderr {
                    execute_ls(ls_args, &mut out_lock, &mut file);
                } else {
                    execute_ls(ls_args, &mut file, &mut err_lock);
                }
            }
            Err(e) => {
                let err_msg = e.to_string();
                let clean_msg = err_msg.split(" (os error").next().unwrap_or(&err_msg);
                eprintln!("ls: {}: {}", file_name, clean_msg);
            }
        }
            return;
        }
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let stderr = io::stderr();
    let mut err = stderr.lock();
    execute_ls(args, &mut out, &mut err);
}

fn execute_ls(args: &[&str], out: &mut impl Write, err: &mut impl Write) {
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
                        Err(e) => {
                            let err_msg = e.to_string();
                            let clean_msg = err_msg.split(" (os error").next().unwrap_or(&err_msg);
                            let _ = writeln!(err, "ls: {}: {}", filename, clean_msg);
                        }
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
            Err(e) => {
                let err_msg = e.to_string();
                let clean_msg = err_msg.split(" (os error").next().unwrap_or(&err_msg);
                let _ = writeln!(err, "ls: {}: {}", filename, clean_msg);
            }
        }
    }

    let _ = out.flush();
    let _ = err.flush();
}