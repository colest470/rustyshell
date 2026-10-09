#[allow(unused_imports)]
use std::io::{self, Write};
use faccess::PathExt;
use pathsearch::PathSearcher;
use std::env;
use std::path::PathBuf;
use std::process::Command;

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

        // if input.trim().split("/").collect::<Vec<_>>()[0] == "." {
        //     let file_path = input.trim().split_whitespace().collect::<Vec<_>>()[0];

        //     let parts = input.trim().split_whitespace().collect::<Vec<_>>();

        //     let args = &parts[1..];

        //     execute_file(file_path, args);
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
            match find_in_path(command) {
                Some(path) => execute_file(path.to_str().unwrap(), args),
                None => println!("{}: command not found", command),
            }

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

fn execute_file(file_path: &str, args: &[&str]) {
    println!("Program was passed {} args (including program name).", );
    // Command::new(file_path)
    //     .args(args)
    //     .spawn()
    //     .expect("Error executing that file");

    // if child.ok {
    //     return true;
    // }

    // return false;
}

fn find_executable(name: &str) -> Option<PathBuf> {
    if name.contains('/') {
        let p = Path::new(name);
        return if p.is_file() && is_executable(p) {
            Some(p.to_path_buf())
        } else {
            None
        };
    }
    let path = env::var_os("PATH")?;
    for dir in env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() && is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}
