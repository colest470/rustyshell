use std::io::{self, Write};

mod builtin;
mod exec;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("No command entered");
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0];
        let args = &parts[1..];

        match command {
            "exit" => break,
            "type" => builtin::builtins(args),
            "echo" => builtin::echo(args),
            _ => match exec::find_in_path(command) {
                Some(path) => exec::execute_file(command, path.to_str().unwrap(), args),
                None => println!("{}: command not found", command),
            }
        }
    }
}
