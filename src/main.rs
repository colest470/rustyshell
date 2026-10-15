use std::io::{self, Write};

mod builtin;
mod exec;
mod helpers;

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

        // Parse standard parts for general commands and path lookups
        let parts: Vec<&str> = input_trimmed.split_whitespace().collect();
        let command = parts[0];
        let args = &parts[1..];

        // Keep a space-split version specifically for echo/cat if needed
        let mut parts_echo: Vec<&str> = input_trimmed.split(' ').collect();

        match command {
            "exit" => break,
            "type" => builtin::builtins(args),
            "echo" => {
                let args_echo = &mut parts_echo[1..];
                builtin::echo(args_echo);
            }
            "cat" => {
                let args_echo = &mut parts_echo[1..];
                builtin::cat(args_echo);
            }
            _ => {
                // Check if it's a direct executable file path (e.g. ./script or /bin/ls)
                let (is_executable_file, exec_command, path_to_file, exec_args) = 
                    helpers::if_executable(input_trimmed);

                if is_executable_file {
                    let final_args: Vec<&str> = exec_args.iter().map(String::as_str).collect();
                    exec::execute_file(&exec_command, &path_to_file, &final_args);
                } else {
                    // Otherwise, search for it in the system PATH
                    match exec::find_in_path(command) {
                        Some(path) => exec::execute_file(command, path.to_str().unwrap(), args),
                        None => println!("{}: command not found", command),
                    }
                }
            }
        }
    }
}