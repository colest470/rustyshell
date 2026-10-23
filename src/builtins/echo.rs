use crate::exec::{append_to_file, write_files};

pub fn echo(input: &str) {
    let remainder = input.strip_prefix("echo").unwrap_or(input).trim_start();
    let args = split_echo_args(remainder);

    let stdout_idx = args.iter().position(|a| a == ">" || a == "1>");
    let stderr_idx = args.iter().position(|a| a == "2>");
    let append_idx = args.iter().position(|a| a == ">>" || a == "1>>");
    let stderr_append_idx = args.iter().position(|a| a == "2>>");

    let first_redirect = [stdout_idx, stderr_idx, append_idx, stderr_append_idx].into_iter().flatten().min();
    let content_end = first_redirect.unwrap_or(args.len());
    let content = args[..content_end].join(" ");

    if let Some(i) = append_idx {
        if i + 1 < args.len() {
            let fname = &args[i + 1];
            if let Err(e) = append_to_file(&format!("{}\n", content), fname) {
                eprintln!("echo: {}: {}", fname, e);
            }
        } else {
            eprintln!("echo: syntax error: expected filename after '>>'");
        }
        return;
    }

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
        if i + 1 >= args.len() {
            eprintln!("echo: syntax error: expected filename after '2>'");
        }
    }

    if let Some(i) = stderr_append_idx {
        if i + 1 >= args.len() {
            eprintln!("echo: syntax error: expected filename after '2>>'");
        }
        // echo writes to stdout; redirecting stderr should not suppress the command output.
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