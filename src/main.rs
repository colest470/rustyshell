#[allow(unused_imports)]
use std::io::{self, Write};
use faccess::PathExt;
use pathsearch::PathSearcher;
use std::env;
use std::path::PathBuf;
use std::Command

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("No command entered");
        let input = input.trim();

        if input.trim().split("/")[0] == "." {
            execute_file(input.trim());
        }

        if input == "exit" {
            break;
        } else if input.split_whitespace().next() == Some("echo") {
            let args: Vec<&str> = input.split_whitespace().skip(1).collect();
            println!("{}", args.join(" "));
        } else if input.split_whitespace().next() == Some("type") {
            let command = input.split_whitespace().nth(1).unwrap_or("");

            if matches!(command, "type" | "exit" | "echo") {
                println!("{} is a shell builtin", command);
            } else if let Some(path) = find_in_path(command) {
                println!("{} is {}", command, path.display());
            } else {
                println!("{}: not found", command);
            }
        } else {
            println!("{}: command not found", input);
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

fn execute_file(path: &str) {
    let mut child = Command::new(path)
        .args()
        .spawn()
        .expect("Error executing that file");

    // if child.ok {
    //     return true;
    // }

    // return false;
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
