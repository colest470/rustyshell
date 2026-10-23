use std::io::{self, Write};
use std::fs;

use crate::helpers::tokenize;

mod exec;
mod helpers;

pub mod builtins;
pub mod builtin;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("No command entered");

        let input_trimmed = input.trim();
        if input_trimmed.is_empty() {
            print!("Invalid command");
            continue;
        }

        let parts: Vec<&str> = input_trimmed.split_whitespace().collect();
        let command = parts[0];
        let args = &parts[1..];

        let tokens = helpers::tokenize(&input);

        match command {
            "exit" => break,
            "type" => builtins::r#type::type_arg(args),
            "echo" => {
                builtins::echo::echo(input_trimmed); // builtin is a folder and echo is a file inside it
            }
            
            "cat" => {
                let tokenized_strings = tokenize(&input);

                let mut stdout_buffer: Vec<u8> = Vec::new();
                let mut stderr_buffer: Vec<u8> = Vec::new();
                let mut append_buffer: Vec<u8> = Vec::new();

                let (mut args_cat, stdout_redirect, stderr_redirect, append_redirect) =
                    helpers::split_redirection_args(&tokenized_strings);

                builtins::cat::cat(
                    &mut args_cat,
                    &mut stdout_buffer,
                    &mut stderr_buffer,
                    &mut append_buffer
                );

                if let Some(filename) = append_redirect {
                    if let Some(parent) = std::path::Path::new(filename).parent() {
                        if !parent.as_os_str().is_empty() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                    }

                    match std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(filename)
                    {
                        Ok(mut file) => {
                            if let Err(e) = file.write_all(&stdout_buffer) {
                                eprintln!("cat: {}: {}", filename, e);
                            }
                        }
                        Err(e) => eprintln!("cat: {}: {}", filename, e),
                    }
                } else if let Some(filename) = stdout_redirect {
                    if let Err(e) = fs::write(filename, &stdout_buffer) {
                        eprintln!("cat: {}: {}", filename, e);
                    }
                } else {
                    let _ = io::stdout().write_all(&stdout_buffer);
                }

                // Handle stderr redirection.
                if let Some(filename) = stderr_redirect {
                    if let Err(e) = fs::write(filename, &stderr_buffer) {
                        eprintln!("cat: {}: {}", filename, e);
                    }
                } else {
                    let _ = io::stderr().write_all(&stderr_buffer);
                }
            }

            "ls" => {
                let mut args_ls: Vec<&str> = tokens.iter().skip(1).map(|s| s.as_str()).collect();
                builtins::ls::ls(&mut args_ls);
            }
            _ => {
                let (is_executable_file, exec_command, path_to_file, exec_args) = 
                    helpers::if_executable(input_trimmed);

                if is_executable_file {
                    let final_args: Vec<&str> = exec_args.iter().map(String::as_str).collect();
                    exec::execute_file(&exec_command, &path_to_file, &final_args);
                } else {
                    match exec::find_in_path(command) {
                        Some(path) => exec::execute_file(command, path.to_str().unwrap(), args),
                        None => println!("{}: command not found", command),
                    }
                }
            }
        }
    }
}