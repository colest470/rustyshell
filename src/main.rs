#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    while 1 {
        repl();
    }
}

fn repl() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut input = String::new();

    io::stdin().read_line(&mut input).expect("No command entered");

    print!("{}: command not found", input.trim());
}
