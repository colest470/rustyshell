use std::io;
use std::io::Write;
use crossterm::{event::{ self, Event, KeyCode, KeyEventKind, KeyModifiers }, terminal};
use std::os::unix::fs::PermissionsExt;
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

// fn complete_path(prefix: &str) -> Option<String> {
//     if prefix.is_empty() {
//         return None;
//     }
//     let path = std::path::Path::new(prefix);
//     let (dir, base) = match path.parent() {
//         Some(p) if !p.as_os_str().is_empty() => (p.to_path_buf(), path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()),
//         _ => (std::path::PathBuf::from("."), prefix.to_string()),
//     };

//     let mut matches = fs::read_dir(&dir)
//         .ok()?
//         .filter_map(Result::ok)
//         .filter(|e| e.file_name().to_string_lossy().starts_with(&base));

//     let first = matches.next()?;
//     if matches.next().is_some() {
//         return None;
//     }

//     let name = first.file_name();
//     let mut out = if prefix.contains('/') {
//         format!("{}/{}", dir.to_string_lossy(), name.to_string_lossy())
//     } else {
//         name.to_string_lossy().into_owned()
//     };

//     let is_dir = first.file_type().map(|t| t.is_dir()).unwrap_or(false);
//     if is_dir {
//         out.push('/');
//     } else {
//         out.push(' ');
//     }
//     Some(out)
// }

fn is_enter_key(key: &event::KeyEvent) -> bool { // find_in_path
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    matches!(key.code, KeyCode::Enter)
        || (ctrl && matches!(key.code, KeyCode::Char('j')))
        || (ctrl && matches!(key.code, KeyCode::Char('m')))
}

pub fn find_in_path_prefix_helper(prefix: &str) -> Vec<std::path::PathBuf> {
    if prefix.is_empty() {
        return Vec::new();
    }
    let path = std::env::var_os("PATH").unwrap_or_default();
    let mut out = Vec::new();
    for dir in std::env::split_paths(&path) {
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for entry in rd.flatten() {
            let name = entry.file_name();
            if !name.to_string_lossy().starts_with(prefix) {
                continue;
            }
            let p = entry.path();
            let Ok(md) = entry.metadata() else { continue };
            if !md.is_file() {
                continue;
            }
            if md.permissions().mode() & 0o111 == 0 {
                continue;
            }
            if !out.contains(&p) {
                out.push(p);
            }
        }
    }
    out
}

fn complete_executable(prefix: &str) -> Option<String> {
    if prefix.is_empty() || prefix.contains(char::is_whitespace) {
        return None;
    }
    let mut matches = find_in_path_prefix_helper(prefix).into_iter();
    match (matches.next(), matches.next()) {
        (Some(only), None) => {
            let name = only
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| only.to_string_lossy().into_owned());
            Some(format!("{} ", name))
        }
        _ => None,
    }
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
                break Ok(None);
            }
            KeyCode::Char('d') if ctrl => {
                if buf.is_empty() {
                    write!(stdout, "\r\n")?;
                    stdout.flush()?;
                    return Ok(None);
                }
            }
            KeyCode::Tab => {
                let word_start = buf
                    .rfind(|c: char| c.is_whitespace())
                    .map(|i| i + 1)
                    .unwrap_or(0);
                let prefix = &buf[word_start..];
                let is_first_word = buf[..word_start].trim().is_empty();

                let completed: Option<String> = if is_first_word {
                    complete(prefix)
                        .or_else(|| complete_executable(prefix))
                        .map(|only| format!("{}{}", &buf[..word_start], only))
                } else {
                    None
                };

                match completed {
                    Some(full) => {
                        // Repaint the whole line: \r, prompt, new buffer.
                        // (Assuming the prompt is "$ " — replace with your real prompt.)
                        write!(stdout, "\r$ {}", full)?;
                        buf = full;
                    }
                    None => write!(stdout, "\x07")?,
                }
            }
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