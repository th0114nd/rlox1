#![feature(let_chains)]
use lazy_static::lazy_static;
use std::fs::File;
use std::io::stdin;
use std::io::BufRead;
use std::io::Write;
use std::io::{self, Read};
use std::sync::Mutex;

mod scanner;
mod token;

use crate::scanner::Scanner;

lazy_static! {
    static ref HAS_FAILURE: Mutex<bool> = Mutex::new(false);
}

fn report(line: usize, r#where: &str, msg: impl AsRef<str>) {
    println!("[line {line}] Error{where}: {}", msg.as_ref());
    set_error();
}

fn set_error() {
    *HAS_FAILURE.lock().unwrap() = true;
}

fn reset_error() {
    *HAS_FAILURE.lock().unwrap() = false;
}

fn error(line: usize, msg: impl AsRef<str>) {
    report(line, "", msg);
}

//fn run(src: String) -> Result<(), Error> {}
fn run(src: &str) -> io::Result<()> {
    let mut scanner = Scanner::new(src);
    let tokens = scanner.scan_tokens();
    for token in tokens {
        println!("{}", token)
    }
    Ok(())
}

fn run_prompt() -> io::Result<()> {
    let mut input = String::new();
    loop {
        input.clear();
        print!("> ");
        io::stdout().flush()?;
        stdin()
            .lock()
            .read_line(&mut input)
            .expect("failed to read line");
        if input.is_empty() {
            return Ok(());
        }
        // TODO: log?
        let _ = run(&input);
        reset_error();
    }
}

fn run_file(file_name: &str) -> io::Result<()> {
    let mut file = File::open(file_name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    run(&contents)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => run_prompt(),
        2 => run_file(&args[1]),
        _ => {
            eprintln!("usage: {} [script]", args[0]);
            std::process::exit(64);
        }
    }
}
