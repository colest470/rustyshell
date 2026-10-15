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

        if input.trim().is_empty() {
            print!("Invalid command");
            continue;
        }

        // a checker to check if the command is for an executable file
        let (is_executable_file, command, path_to_file, args) = helpers::if_executable(input.trim());

        print!("{:?}", is_executable_file);

        if is_executable_file {
            let final_args: Vec<&str> = args.iter().map(String::as_str).collect();
            exec::execute_file(command.as_str(), &path_to_file, &final_args);
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0];
        let args = &parts[1..];

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
