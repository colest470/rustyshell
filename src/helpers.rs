
pub fn remove_trailing_new_line<'a>(args: &'a mut [&'a str])-> &'a mut [&'a str] {
    let args_length: usize = args.len();
    let mut last_element: &str = args[args_length - 1];
    last_element = &last_element[..last_element.len() - 1];
    args[args_length - 1] = last_element;

    return args;
}

pub fn unquote(arg: &str) -> String {
    let mut result = String::new();
    let mut in_quotes = false;

    for ch in arg.chars() {
        match ch {
            '\'' => in_quotes = !in_quotes,
            _ => {
                if in_quotes {
                    result.push(ch);
                }
            }
        }
    }

    if !arg.contains('\'') {
        return arg.to_string();
    }

    return result;
}
