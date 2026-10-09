use faccess::PathExt;
use pathsearch::PathSearcher;
use std::env;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

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
