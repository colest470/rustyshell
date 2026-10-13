

pub fn remove_trailing_new_line<'a>(args: &'a mut [&'a str])-> &'a mut [&'a str] {
    let args_length: usize = args.len();
    let mut last_element: &str = args[args_length - 1];
    last_element = &last_element[..last_element.len() - 1];
    args[args_length - 1] = last_element;

    return args;
}


pub fn unquote<'a>(arg: &'a str) -> Vec<&'a str> {
    let mut result: Vec<&'a str> = Vec::new();
    let mut in_quotes = false;
    let mut segment_start = 0usize;

    for (i, ch) in arg.char_indices() {
        match ch {
            '\'' => {
                let segment = &arg[segment_start..i];
                if !segment.is_empty() {
                    result.push(segment);
                }
                in_quotes = !in_quotes;
                segment_start = i + 1;
            }
            _ => {}
        }
    }

    if segment_start < arg.len() {
        let segment = &arg[segment_start..];
        if !segment.is_empty() {
            result.push(segment);
        }
    }

    result
}