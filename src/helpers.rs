pub fn remove_trailing_new_line<'a, 'b>(args: &'a mut [&'b str]) -> &'a mut [&'b str] {
    let args_length: usize = args.len();
    let mut last_element: &str = args[args_length - 1];
    last_element = &last_element[..last_element.len() - 1];
    args[args_length - 1] = last_element;

    return args;
}


pub fn unquote(arg: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut has_content = false;
    let mut chars = arg.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\'' if !in_double => {
                in_single = !in_single;
                has_content = true;
            }
            '"' if !in_single => {
                in_double = !in_double;
                has_content = true;
            }
            '\\' if !in_single => {
                if let Some(next) = chars.next() {
                    if in_double && !matches!(next, '"' | '\\' | '$' | '`' | '\n') {
                        current.push('\\');
                        current.push(next);
                    } else {
                        current.push(next);
                    }
                    has_content = true;
                } else {
                    current.push('\\');
                    has_content = true;
                }
            }
            ' ' | '\t' if !in_single && !in_double => {
                if has_content {
                    result.push(std::mem::take(&mut current));
                    has_content = false;
                }
            }
            _ => {
                current.push(ch);
                has_content = true;
            }
        }
    }

    if has_content {
        result.push(current);
    }

    result
}

pub fn check_if_executable(input: &str) -> (bool, ) {
    let parts = unquote(input);

    print!("{:?}", parts);

    return (false, );
}