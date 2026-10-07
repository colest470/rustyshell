#[allow(unused_imports)]
use std::io::{self, Write};

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
            let command = input.trim().split(" ").collect::<Vec<_>>()[1];

            if command.contains(&"type") || command.contains(&"exit") || command.contains(&"echo") {
                print!("{} is a shell builtin\n", command.trim());
            } else {
                print!("{}: not found\n", command.trim());
            }
        } else {
            print!("{}: command not found\n", input.trim());
        }
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
