use std::{fs, io};
// use std::io;
use std::io::Write;
use crossterm::{event::{ self, Event, KeyCode, KeyEventKind, KeyModifiers }, terminal};

use crate::exec::find_in_path;

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

// pub fn read_file(path: &str) -> io::Result<String> {
//     let content = fs::read_to_string(path)?;
//     Ok(content)
// }

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

pub fn split_redirection_args(
    tokens: &[String],
) -> (Vec<&str>, Option<(&str, bool)>, Option<(&str, bool)>) {
    let mut args = Vec::new();
    let mut stdout_redirect: Option<(&str, bool)> = None;
    let mut stderr_redirect: Option<(&str, bool)> = None;
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i].as_str() {
            ">"  | "1>"  => { if i + 1 < tokens.len() { stdout_redirect = Some((tokens[i+1].as_str(), false)); } i += 2; }
            ">>" | "1>>" => { if i + 1 < tokens.len() { stdout_redirect = Some((tokens[i+1].as_str(), true));  } i += 2; }
            "2>"         => { if i + 1 < tokens.len() { stderr_redirect = Some((tokens[i+1].as_str(), false)); } i += 2; }
            "2>>"        => { if i + 1 < tokens.len() { stderr_redirect = Some((tokens[i+1].as_str(), true));  } i += 2; }
            token        => { args.push(token); i += 1; }
        }
    }

    (args, stdout_redirect, stderr_redirect)
}

pub fn write_redirected(
    cmd_name: &str,
    target: Option<(&str, bool)>,
    buf: &[u8],
    default: impl FnOnce(&[u8]),
) {
    match target {
        Some((filename, append)) => {
            if let Some(parent) = std::path::Path::new(filename).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = fs::create_dir_all(parent);
                }
            }

            let mut o = fs::OpenOptions::new();
            o.create(true);
            if append {
                o.append(true);
            } else {
                o.write(true).truncate(true);
            }

            match o.open(filename) {
                Ok(mut f) => {
                    if let Err(e) = f.write_all(buf) {
                        eprintln!("{}: {}: {}", cmd_name, filename, e);
                    }
                }
                Err(e) => eprintln!("{}: {}: {}", cmd_name, filename, e),
            }
        }
        None => default(buf),
    }
}
struct RawModeGuard;

impl RawModeGuard {
    fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

const BUILTINS: &[&str] = &["echo", "exit", "type", "cat", "ls"];

fn complete(prefix: &str) -> Option<String> {
    if prefix.is_empty() || prefix.contains(char::is_whitespace) {
        return None;
    }
    let mut matches = BUILTINS.iter().filter(|c| c.starts_with(prefix));
    match (matches.next(), matches.next()) {
        (Some(only), None) => Some(format!("{only} ")),
        _ => None,
    }
}

fn is_enter_key(key: &event::KeyEvent) -> bool {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    matches!(key.code, KeyCode::Enter)
        || (ctrl && matches!(key.code, KeyCode::Char('j')))
        || (ctrl && matches!(key.code, KeyCode::Char('m')))
}

pub fn read_line_interactive() -> io::Result<Option<String>> {
    let _guard = RawModeGuard::new()?;
    let mut stdout = io::stdout();
    let mut buf = String::new();

    loop {
        let Event::Key(key) = event::read()? else {
            continue;
        };

        if key.kind != KeyEventKind::Press {
            continue;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let is_enter = is_enter_key(&key);

        if is_enter {
            write!(stdout, "\r\n")?;
            stdout.flush()?;
            return Ok(Some(buf));
        }

        match key.code {
            KeyCode::Char('c') if ctrl => {
                write!(stdout, "^C\r\n")?;
                stdout.flush()?;
                return Ok(Some(String::new()));
            }
            KeyCode::Char('d') if ctrl => {
                if buf.is_empty() {
                    write!(stdout, "\r\n")?;
                    stdout.flush()?;
                    return Ok(None);
                }
            }
            KeyCode::Tab => match complete(&buf) {
                Some(full) => {
                    write!(stdout, "{}", &full[buf.len()..])?;
                    buf = full;
                }
                None => write!(stdout, "\x07")?,
            },
            KeyCode::Backspace => {
                if buf.pop().is_some() {
                    write!(stdout, "\x08 \x08")?;
                }
            }
            KeyCode::Char(c) if !ctrl => {
                buf.push(c);
                write!(stdout, "{c}")?;
            }
            _ => {}
        }
        stdout.flush()?;
    }
}