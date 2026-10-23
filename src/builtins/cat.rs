use std::io::{self, Read, Write};
use std::fs::{File};

use crate::helpers::{remove_trailing_new_line};

fn clean_io(e: &io::Error) -> String {
    let s = e.to_string();
    match s.find(" (os error ") {
        Some(i) => s[..i].to_string(),
        None => s,
    }
}

pub fn cat(args: &mut [&str], out: &mut impl Write, err: &mut impl Write, _append: &mut impl Write) {
    remove_trailing_new_line(args);

    let rest = if args.first().is_some_and(|arg| *arg == "cat") {
        &args[1..]
    } else {
        args
    };

    if rest.is_empty() {
        let stdin = io::stdin();
        let mut lock = stdin.lock();
        let mut buf = [0u8; 8192];

        loop {
            match lock.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if out.write_all(&buf[..n]).is_err() {
                        return;
                    }
                }
                Err(e) => {
                    let _ = writeln!(err, "cat: {}", clean_io(&e));
                    break;
                }
            }
        }

        let _ = out.flush();
        let _ = err.flush();
        return;
    }

    let mut paths: Vec<&str> = Vec::new();
    let mut i = 0;

    while i < rest.len() {
        match rest[i] {
            ">" | "1>" | "2>" | ">>" | "1>>" => {
                break;
            }
            path => {
                paths.push(path);
                i += 1;
            }
        }
    }

    for path in paths {
        match File::open(path) {
            Ok(mut file) => {
                let mut buf = [0u8; 8192];

                loop {
                    match file.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if out.write_all(&buf[..n]).is_err() {
                                return;
                            }
                        }
                        Err(e) => {
                            let _ = writeln!(err,"cat: nonexistent: {}",clean_io(&e));
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                let _ = writeln!(
                    err,
                    "cat: nonexistent: {}",
                    clean_io(&e)
                );
            }
        }
    }

    let _ = out.flush();
    let _ = err.flush();
}