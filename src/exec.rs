use faccess::PathExt;
use pathsearch::PathSearcher;
use std::env;
use std::fs::OpenOptions;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::io::{self, Write};

pub fn find_in_path(command: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH");
    let path_ext = env::var_os("PATHEXT");

    let candidates = PathSearcher::new(command, path.as_deref(), path_ext.as_deref());

    for candidate in candidates {
        if candidate.is_file() && candidate.executable() {
            return Some(candidate);
        }
    }
    None
}

pub fn execute_file(command: &str, file_path: &str, args: &[&str]) {
    Command::new(file_path)
        .arg0(command)
        .args(args)
        .status()
        .expect("Error executing that file");
}

pub fn write_files(content: &str, path: &str) -> std::io::Result<()> {
    let mut f = OpenOptions::new().write(true).create(true).truncate(true).open(path)?;
    f.write_all(content.as_bytes())
}

pub fn append_to_file(content: &str, path: &str) -> std::io::Result<()> {
    let mut f = OpenOptions::new().append(true).create(true).open(path)?;
    f.write_all(content.as_bytes())
}