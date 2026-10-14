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
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut has_content = false; 

    for ch in arg.chars() {
        match ch {
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
                has_content = true;
            }
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
                has_content = true;
            }
            ' ' | '\t' if !in_single_quotes && !in_double_quotes => {
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