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
    let args = split_echo_args(remainder);

    // Find stdout redirect
    let stdout_idx = args.iter().position(|a| a == ">" || a == "1>");
    // Find stderr redirect
    let stderr_idx = args.iter().position(|a| a == "2>");

    // Content = everything up to the first redirect operator
    let first_redirect = [stdout_idx, stderr_idx].into_iter().flatten().min();
    let content_end = first_redirect.unwrap_or(args.len());
    let content = args[..content_end].join(" ");

    // Handle stdout redirect: write to file and return
    if let Some(i) = stdout_idx {
        if i + 1 < args.len() {
            let fname = &args[i + 1];
            if let Err(e) = write_files(&format!("{}\n", content), fname) {
                eprintln!("echo: {}: {}", fname, e);
            }
        } else {
            eprintln!("echo: syntax error: expected filename after '>'");
        }
        return;
    }

    if let Some(i) = stderr_idx {
        if i + 1 < args.len() {
            let fname = &args[i + 1];
            if let Err(e) = write_files("", fname) {
                eprintln!("echo: {}: {}", fname, e);
            }
        } else {
            eprintln!("echo: syntax error: expected filename after '2>'");
        }
    }

    if content.is_empty() {
        println!();
    } else {
        println!("{}", content);
    }
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


pub fn cat(args: &mut [&str], out: &mut impl Write, err: &mut impl Write) {
    remove_trailing_new_line(args);

    let rest = &args[1..];

    if rest.is_empty() {
        let stdin = io::stdin();
        let mut lock = stdin.lock();
        let mut buf = [0u8; 8192];

        loop {
            match lock.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if out.write_all(&buf[..n]).is_err() {
                        return;
                    }
                }
                Err(e) => {
                    let _ = writeln!(err, "cat: {}", clean_io(&e));
                    break;
                }
            }
        }

        let _ = out.flush();
        let _ = err.flush();
        return;
    }

    let mut paths: Vec<&str> = Vec::new();
    let mut i = 0;

    while i < rest.len() {
        match rest[i] {
            ">" | "1>" | "2>" => {
                break;
            }
            path => {
                paths.push(path);
                i += 1;
            }
        }
    }

    for path in paths {
        match File::open(path) {
            Ok(mut file) => {
                let mut buf = [0u8; 8192];

                loop {
                    match file.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if out.write_all(&buf[..n]).is_err() {
                                return;
                            }
                        }
                        Err(e) => {
                            let _ = writeln!(err,"cat: nonexistent: {}",clean_io(&e));
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                let _ = writeln!(
                    err,
                    "cat: nonexistent: {}",
                    clean_io(&e)
                );
            }
        }
    }

    let _ = out.flush();
    let _ = err.flush();
}


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