use std::fs;
use std::io;

use crate::exec::{find_in_path};

pub fn remove_trailing_new_line<'a, 'b>(args: &'a mut [&'b str]) -> &'a mut [&'b str] {
    if let Some(last) = args.last_mut() {
        if last.ends_with('\n') {
            *last = &last[..last.len() - 1];
        }
        if last.ends_with('\r') {
            *last = &last[..last.len() - 1];
        }
    }
    args
}
#[cfg(test)]
mod tests {
    use super::tokenize;

    #[test]
    fn tokenize_removes_trailing_newline_from_paths() {
        let tokens = tokenize("cat /tmp/dog/fox.md\n");
        assert_eq!(tokens, vec!["cat".to_string(), "/tmp/dog/fox.md".to_string()]);
    }

    #[test]
    fn tokenize_keeps_quoted_paths_with_spaces() {
        let tokens = tokenize("cat /tmp/cow/\"number 60\" /tmp/cow/\"doublequote \\\" 28\"\n");
        assert_eq!(
            tokens,
            vec![
                "cat".to_string(),
                "/tmp/cow/number 60".to_string(),
                "/tmp/cow/doublequote \" 28".to_string(),
            ]
        );
    }

    #[test]
    fn tokenize_keeps_backslashes_in_quoted_paths() {
        let tokens = tokenize("cat /tmp/cow/\"backslash \\\\ 24\"\n");
        assert_eq!(
            tokens,
            vec![
                "cat".to_string(),
                "/tmp/cow/backslash \\ 24".to_string(),
            ]
        );
    }

    #[test]
    fn tokenize_handles_append_redirect() {
        let tokens = tokenize("ls -1 /tmp/ant >> /tmp/dog/bee.md\n");
        assert_eq!(
            tokens,
            vec![
                "ls".to_string(),
                "-1".to_string(),
                "/tmp/ant".to_string(),
                ">>".to_string(),
                "/tmp/dog/bee.md".to_string(),
            ]
        );
    }
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

pub fn if_executable(input: &str) -> (bool, String, String, Vec<String>) {
    let parts = unquote(input);

    let opt = find_in_path(parts[0].as_str());

    if let Some(path) = opt {
        let command = parts[0].clone();
        let path_to_file = path.to_str().unwrap().to_string();
        let args: Vec<String> = parts[1..].to_vec();

        return (true, command, path_to_file, args);
    }

    (false, String::new(), String::new(), Vec::new())
}

pub fn read_file(path: &str) -> io::Result<String> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}

pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut started = false;
    let mut in_single = false;
    let mut in_double = false;
    let mut chars = input.chars().peekable();

    // Helper closure to flush the current token
    let flush = |tokens: &mut Vec<String>, cur: &mut String, started: &mut bool| {
        if *started || !cur.is_empty() {
            let mut value = std::mem::take(cur);
            while value.ends_with('\n') || value.ends_with('\r') {
                value.pop();
            }
            tokens.push(value);
            *started = false;
        }
    };

    while let Some(c) = chars.next() {
        if in_single {
            if c == '\'' {
                in_single = false;
            } else {
                cur.push(c);
            }
            started = true;
        } else if in_double {
            match c {
                '"' => { in_double = false; }
                '\\' => match chars.next() {
                    Some(n @ ('"' | '\\' | '$' | '`' | '\n')) => cur.push(n),
                    Some(n) => { cur.push('\\'); cur.push(n); }
                    None    => cur.push('\\'),
                },
                _ => cur.push(c),
            }
            started = true;
        } else {
            match c {
                '\'' => { in_single = true; started = true; }
                '"'  => { in_double = true; started = true; }
                '\\' => {
                    if let Some(n) = chars.next() { cur.push(n); }
                    started = true;
                }
                d @ ('0'..='9') => {
                    if matches!(chars.peek(), Some('>') | Some('<')) {
                        flush(&mut tokens, &mut cur, &mut started);
                        let op = chars.next().unwrap();
                        if matches!(chars.peek(), Some('>')) && op == '>' {
                            chars.next();
                            tokens.push(format!("{}>>", d));
                        } else {
                            tokens.push(format!("{}{}", d, op));
                        }
                    } else {
                        cur.push(d);
                        started = true;
                    }
                }
                '>' => {
                    flush(&mut tokens, &mut cur, &mut started);
                    if matches!(chars.peek(), Some('>')) {
                        chars.next();
                        tokens.push(">>".to_string());
                    } else {
                        tokens.push(">".to_string());
                    }
                }
                '<' => {
                    flush(&mut tokens, &mut cur, &mut started);
                    tokens.push("<".to_string());
                }
                ' ' | '\t' => {
                    if started {
                        tokens.push(std::mem::take(&mut cur));
                        started = false;
                    }
                }
                _ => { cur.push(c); started = true; }
            }
        }
    }
    flush(&mut tokens, &mut cur, &mut started);
    tokens
}

pub fn split_redirection_args(tokens: &[String]) -> (Vec<&str>, Option<&str>, Option<&str>, Option<&str>) {
    let mut args = Vec::new();
    let mut stdout_redirect = None;
    let mut stderr_redirect = None;
    let mut append_redirect = None;
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i].as_str() {
            ">" | "1>" => {
                if i + 1 < tokens.len() {
                    stdout_redirect = Some(tokens[i + 1].as_str());
                }
                i += 2;
            }
            "2>" => {
                if i + 1 < tokens.len() {
                    stderr_redirect = Some(tokens[i + 1].as_str());
                }
                i += 2;
            }
            "2>>" => {
                if i + 1 < tokens.len() {
                    append_redirect = Some(tokens[i + 1].as_str());
                }
                i += 2;
            }
            ">>" | "1>>" => {
                if i + 1 < tokens.len() {
                    append_redirect = Some(tokens[i + 1].as_str());
                }
                i += 2;
            }
            token => {
                args.push(token);
                i += 1;
            }
        }
    }

    (args, stdout_redirect, stderr_redirect, append_redirect)
}