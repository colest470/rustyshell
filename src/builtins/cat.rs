use std::fs::File;
use std::io::{self, Read, Write};

use crate::helpers::remove_trailing_new_line;

fn clean_io(e: &io::Error) -> String {
    let s = e.to_string();
    match s.find(" (os error ") {
        Some(i) => s[..i].to_string(),
        None => s,
    }
}

fn pump(src: &mut impl Read, out: &mut impl Write) -> io::Result<()> {
    let mut buf = [0u8; 8192];
    loop {
        match src.read(&mut buf)? {
            0 => return Ok(()),
            n => out.write_all(&buf[..n])?,
        }
    }
}

pub fn cat(args: &mut [&str], out: &mut impl Write, err: &mut impl Write, _append: &mut impl Write) {
    remove_trailing_new_line(args);

    let rest = if args.first().is_some_and(|a| *a == "cat") { &args[1..] } else { args };

    let paths: Vec<&str> = rest
        .iter()
        .copied()
        .take_while(|a| !matches!(*a, ">" | "1>" | "2>" | ">>" | "1>>" | "2>>"))
        .collect();

    if paths.is_empty() {
        if let Err(e) = pump(&mut io::stdin().lock(), out) {
            let _ = writeln!(err, "cat: {}", clean_io(&e));
        }
    } else {
        for path in paths {
            match File::open(path) {
                Ok(mut file) => {
                    if let Err(e) = pump(&mut file, out) {
                        let _ = writeln!(err, "cat: {}: {}", path, clean_io(&e));
                    }
                }
                Err(e) => {
                    let _ = writeln!(err, "cat: {}: {}", path, clean_io(&e));
                }
            }
        }
    }

    let _ = out.flush();
    let _ = err.flush();
}