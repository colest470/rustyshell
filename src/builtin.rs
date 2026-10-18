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
    // 1. Detect redirection: `> file` or `1> file`
    let redirect_pos = args
        .iter()
        .position(|&x| x == ">" || x == "1>");

    let (ls_args, dest): (&[&str], Option<&str>) = match redirect_pos {
        Some(i) => {
            if i + 1 >= args.len() {
                eprintln!("ls: syntax error: expected filename after '>'");
                return;
            }
            (&args[..i], Some(args[i + 1]))
        }
        None => (&args[..], None),
    };

    // 2. Parse flags
    let mut long_format = false;
    let mut one_per_line = false;
    let mut show_all = false;
    let mut paths: Vec<&str> = Vec::new();

    for arg in ls_args.iter() {
        match *arg {
            "-1" => one_per_line = true,
            "-a" => show_all = true,
            "-l" => long_format = true,
            _ if arg.starts_with('-') => {
                // combined flags like `-la`
                for ch in arg.chars().skip(1) {
                    match ch {
                        '1' => one_per_line = true,
                        'a' => show_all = true,
                        'l' => long_format = true,
                        _ => {}
                    }
                }
            }
            _ => paths.push(arg),
        }
    }

    if paths.is_empty() {
        paths.push(".");
    }

    // 3. Build output into a buffer (do NOT print directly)
    let mut out = String::new();
    let multiple_paths = paths.len() > 1;

    for (idx, path) in paths.iter().enumerate() {
        let meta = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                let msg = e.to_string();
                let clean = msg.split(" (os error").next().unwrap_or(&msg);
                eprintln!("ls: {path}: {clean}");
                continue;
            }
        };

        if multiple_paths {
            if idx > 0 {
                out.push('\n');
            }
            out.push_str(path);
            out.push_str(":\n");
        }

        // Single file: just print its name
        if meta.is_file() {
            out.push_str(path);
            out.push('\n');
            continue;
        }

        // Directory listing
        let entries = match fs::read_dir(path) {
            Ok(e) => e,
            Err(e) => {
                let msg = e.to_string();
                let clean = msg.split(" (os error").next().unwrap_or(&msg);
                eprintln!("ls: {path}: {clean}");
                continue;
            }
        };

        let mut names: Vec<String> = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !show_all && name.starts_with('.') {
                continue;
            }
            names.push(name);
        }

        // ls sorts alphabetically by default
        names.sort();

        if long_format {
            // Re-read metadata for each entry to print sizes etc.
            // (We already sorted names, so re-walk the dir via names.)
            // Build long-format lines into `out`.
            for name in &names {
                let full = std::path::Path::new(path).join(name);
                match fs::metadata(&full) {
                    Ok(m) => {
                        let size = if m.is_dir() { 0 } else { m.len() };
                        let kind = if m.is_dir() { 'd' } else { '-' };
                        out.push_str(&format!(
                            "{}{} {} {}\n",
                            kind,
                            // simple perms placeholder; extend if tests need real mode bits
                            "rw-r--r--",
                            size,
                            name
                        ));
                    }
                    Err(_) => {
                        out.push_str(name);
                        out.push('\n');
                    }
                }
            }
        } else if one_per_line {
            for name in &names {
                out.push_str(name);
                out.push('\n');
            }
        } else {
            // Default: space-separated columns on one line.
            out.push_str(&names.join("  "));
            if !names.is_empty() {
                out.push('\n');
            }
        }
    }

    // 4. Emit
    if let Some(dest) = dest {
        if let Some(parent) = std::path::Path::new(dest).parent() {
            if !parent.as_os_str().is_empty() {
                let _ = fs::create_dir_all(parent);
            }
        }
        if let Err(e) = write_files(&out, dest) {
            let msg = e.to_string();
            let clean = msg.split(" (os error").next().unwrap_or(&msg);
            eprintln!("ls: {dest}: {clean}");
        }
    } else {
        print!("{out}");
    }
}