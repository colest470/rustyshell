#[allow(unused_imports)]
use std::io::{self, Write};
use faccess::PathExt;
use pathsearch::PathSearcher;

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
            } else if let Some(path) = find_in_path(command_string) {
                println!("{} is {}", command_string, path.display());
            } else {
                print!("{}: not found\n", command_string.trim());
            }
        } else {
            print!("{}: command not found\n", input.trim());
        }
    }
}

fn find_in_path(command: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH");
    let path_ext = env::var_os("PATHEXT");

    let candidates = PathSearcher::new(command, path.as_deref(), path_ext.as_deref());

    for candidate in candidates {
        if candidate.is_file() && candidate.executable() {
            return Some(candidate);
        }
    }

    None
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
