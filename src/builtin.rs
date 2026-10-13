use crate::exec::find_in_path;

pub fn builtins (args: &[&str]) {
    let target = args.first().copied().unwrap_or("");

    if matches!(target, "type" | "exit" | "echo") {
        println!("{} is a shell builtin", target);
    } else if let Some(path) = find_in_path(target) {
        println!("{} is {}", target, path.display());
    } else {
        println!("{}: not found", target);
    }
}

pub fn echo(args: &mut [&str]) {
    // Remove the trailing newline from the last element
    args = remove_trailing_new_line(args)

    let input = args.join(" ");
    
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    let mut in_quotes = false;
    let mut last_was_space = false;
    
    while let Some(ch) = chars.next() {
        match ch {
            '\'' => {
                in_quotes = !in_quotes;
            }
            ' ' => {
                if in_quotes {
                    result.push(ch);
                    last_was_space = true;
                } else {
                    if !last_was_space && !result.is_empty() {
                        result.push(' ');
                        last_was_space = true;
                    }
                }
            }
            _ => {
                result.push(ch);
                last_was_space = false;
            }
        }
    }
    
    println!("{}", result);
}

pub fn cat() {

}

pb fn remove_trailing_new_line (args: &mut [&str])-> &mut [&str] {
    let args_length = args.len();
    let mut last_element = args[args_length - 1];
    last_element = &last_element[..last_element.len() - 1];
    args[args_length - 1] = last_element;

    return args;
}