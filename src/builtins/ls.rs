use std::io::{self, Write};
use std::fs::{self, File, OpenOptions};
use std::path::Path;

use crate::helpers::{remove_trailing_new_line, unquote};

pub fn ls(args: &mut [&str]) {
    remove_trailing_new_line(args);

    let mut redirect_idx = None;
    let mut is_stderr = false;
    let mut is_append = false;

    for (i, element) in args.iter().enumerate() {
        if *element == ">" || *element == "1>" {
            redirect_idx = Some(i);
            is_stderr = false;
            is_append = false;
            break;
        } else if *element == "2>" {
            redirect_idx = Some(i);
            is_stderr = true;
            is_append = false;
            break;
        } else if *element == ">>" || *element == "1>>" {
            redirect_idx = Some(i);
            is_stderr = false;
            is_append = true;
            break;
        } else if *element == "2>>" {
            redirect_idx = Some(i);
            is_stderr = true;
            is_append = true;
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

            let open_result = if is_append {
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(true)
                    .open(file_name)
            } else {
                File::create(file_name)
            };

            match open_result {
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