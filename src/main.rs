use std::io::{self, Write};

mod builtin;
mod exec;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("No command entered");

        if input.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0];
        let args = &parts[1..];

        // 1. Make the vector mutable so we can take mutable slices from it
        let mut parts_echo: Vec<&str> = input.split(' ').collect();

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
            _ => match exec::find_in_path(command) {
                Some(path) => exec::execute_file(command, path.to_str().unwrap(), args),
                None => println!("{}: command not found", command),
            }
        }
    }
}
