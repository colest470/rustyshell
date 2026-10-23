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

use std::fs::OpenOptions;

enum Sink {
    Stdout,
    Stderr,
    File(std::fs::File),
}

impl Write for Sink {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        match self {
            Sink::Stdout => io::stdout().write(b),
            Sink::Stderr => io::stderr().write(b),
            Sink::File(f) => f.write(b),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self {
            Sink::Stdout => io::stdout().flush(),
            Sink::Stderr => io::stderr().flush(),
            Sink::File(f) => f.flush(),
        }
    }
}

fn open_sink(path: &str, append: bool) -> io::Result<Sink> {
    let mut o = OpenOptions::new();
    o.create(true);
    if append { o.append(true); } else { o.write(true).truncate(true); }
    Ok(Sink::File(o.open(path)?))
}

fn resolve_redirects<'a>(args: &[&'a str]) -> io::Result<(Vec<&'a str>, Sink, Sink)> {
    let mut cmd = Vec::new();
    let mut out = Sink::Stdout;
    let mut err = Sink::Stderr;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            op @ (">" | "1>" | ">>" | "1>>" | "2>" | "2>>") if i + 1 < args.len() => {
                let append = op.ends_with(">>");
                let sink = open_sink(args[i + 1], append)?;
                if op.starts_with('2') { err = sink } else { out = sink }
                i += 2;
            }
            a => { cmd.push(a); i += 1; }
        }
    }
    Ok((cmd, out, err))
}

pub fn cat(args: &mut [&str], out: &mut impl Write, err: &mut impl Write) {
    remove_trailing_new_line(args);
    let rest = if args.first().is_some_and(|a| *a == "cat") { &args[1..] } else { args };

    if rest.is_empty() {
        if let Err(e) = pump(&mut io::stdin().lock(), out) {
            let _ = writeln!(err, "cat: {}", clean_io(&e));
        }
    } else {
        for path in rest {
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