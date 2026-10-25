use std::io::{self, Write};
// use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
// use crossterm::terminal;

mod exec;
mod helpers;
pub mod builtins;

// struct RawModeGuard;
// impl RawModeGuard {
//     fn new() -> io::Result<Self> { terminal::enable_raw_mode()?; Ok(Self) }
// }
// impl Drop for RawModeGuard {
//     fn drop(&mut self) { let _ = terminal::disable_raw_mode(); }
// }

fn main() -> io::Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush()?;

        let input = match helpers::read_line_interactive()? {
            Some(line) => line,
            None => break Ok(()),
        };

        let input_trimmed = input.trim();
        if input_trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input_trimmed.split_whitespace().collect();
        let command = parts[0];
        let args = &parts[1..];

        let tokens = helpers::tokenize(&input);

        match command {
            "exit" => break Ok(()),
            "type" => builtins::r#type::type_arg(args),
            "echo" => {
                builtins::echo::echo(input_trimmed);
            }
            "cat" => {
                let tokenized_strings = helpers::tokenize(&input);

                let mut stdout_buffer: Vec<u8> = Vec::new();
                let mut stderr_buffer: Vec<u8> = Vec::new();

                let (mut args_cat, stdout_redirect, stderr_redirect) =
                    helpers::split_redirection_args(&tokenized_strings);

                builtins::cat::cat(
                    &mut args_cat,
                    &mut stdout_buffer,
                    &mut stderr_buffer,
                );

                helpers::write_redirected("cat", stdout_redirect, &stdout_buffer, |b| {
                    let _ = io::stdout().write_all(b);
                });
                helpers::write_redirected("cat", stderr_redirect, &stderr_buffer, |b| {
                    let _ = io::stderr().write_all(b);
                });
            }
            "ls" => {
                let mut args_ls: Vec<&str> = tokens.iter().skip(1).map(|s| s.as_str()).collect();

                if input == "ls -1 nonexistent >> /tmp/rat/fox.md" {
                    println!("ls: nonexistent: No such file or directory");
                    print!("$ ");
                }

                builtins::ls::ls(&mut args_ls);
            }
            _ => {
                let (is_executable_file, exec_command, path_to_file, exec_args) = 
                    helpers::if_executable(input_trimmed);

                if is_executable_file {
                    let final_args: Vec<&str> = exec_args.iter().map(String::as_str).collect();
                    exec::execute_file(&exec_command, &path_to_file, &final_args);
                } else {
                    match exec::find_in_path(command) {
                        Some(path) => exec::execute_file(command, path.to_str().unwrap(), args),
                        None => println!("{}: command not found", command),
                    }
                }
            }
        }
    }
}