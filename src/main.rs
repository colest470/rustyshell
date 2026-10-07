#[allow(unused_imports)]
use std::io::{self, Write};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();

        io::stdin().read_line(&mut input).expect("No command entered");

        if input.trim() == "exit"{
            break;
        } else if input.split_whitespace().next() == Some("echo") {

            let output_echo_output: Vec<&str> = input.trim().split(" ").skip(1).collect();
            println!("{}", output_echo_output.join(" "));
        } else if input.split_whitespace().next() == Some("type") {
            let command_string = input.trim().split_whitespace().nth(1).unwrap_or("");

            if command_string.contains(&"type") || command_string.contains(&"exit") || command_string.contains(&"echo") {
                print!("{} is a shell builtin\n", command_string.trim());
            } else if is_executable(&(PathBuf::from("/usr/bin/").join(command_string))){
                print!("{} is /usr/bin/{}\n", command_string.trim(), command_string.trim());
            } else {
                print!("{}: not found\n", command_string.trim());
            }
        } else {
            print!("{}: command not found\n", input.trim());
        }
    }
}

fn is_executable(path: &Path) -> bool {
    match fs::metadata(path) {
        Ok(meta) => meta.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}

// fn repl()-> i8 {
//     print!("$ ");
//     io::stdout().flush().unwrap();

//     let mut input = String::new();

//     io::stdin().read_line(&mut input).expect("No command entered");

//     let something: i8 = 1;

//     if input == "exit" {
//         return something;
//     }

//     print!("{}: command not found\n", input.trim());
// }
