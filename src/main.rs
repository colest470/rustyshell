#[allow(unused_imports)]
use std::io::{self, Write};
use faccess::PathExt;
use pathsearch::PathSearcher;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::os::unix::process::CommandExt;

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
            "type" => {
                let target = args.first().copied().unwrap_or("");

                if matches!(target, "type" | "exit" | "echo") {
                    println!("{} is a shell builtin", target);
                } else if let Some(path) = find_in_path(target) {
                    println!("{} is {}", target, path.display());
                } else {
                    println!("{}: not found", target);
                }
            }
            _ => match find_in_path(command) {
                Some(path) => execute_file(command, path.to_str().unwrap(), args),
                None => println!("{}: command not found", command),
            }
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

fn execute_file(command: &str, file_path: &str, args: &[&str]) {
    Command::new(file_path)
        .arg0(command)
        .args(args)
        .status()
        .expect("Error executing that file");
}