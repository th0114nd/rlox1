#![feature(hash_raw_entry)]
#![feature(let_chains)]
use std::fs::File;
use std::io;
use std::io::stdin;
use std::io::BufRead;
use std::io::Read;
use std::io::Write;

mod callable;
mod environment;
mod error;
mod expr;
mod expr_eval;
mod interpreter;
mod models;
mod parser;
mod scanner;
mod stmt;
mod stmt_eval;
mod token;
mod value;

use crate::error::MainError;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::scanner::Scanner;

//use thiserror::Error;

//#[derive(Debug, Error)]
//enum MainError {
//    #[error("io error: {0}")]
//    IoError(#[from] io::Error),
//    #[error("runtime error: {0}")]
//    MainError(#[from] MainError),
//}

type MainResult = Result<(), MainError>;

fn run(int: &mut Interpreter, src: &str) -> MainResult {
    let mut scanner = Scanner::new(src);
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(&tokens);
    let stmts = parser.parse()?;

    Ok(int.interpret(stmts).map(move |_| ())?)
}

fn run_prompt(int: &mut Interpreter) -> MainResult {
    let mut input = String::new();
    loop {
        input.clear();
        print!("> ");
        io::stdout().flush()?;
        stdin().lock().read_line(&mut input)?;
        if input.is_empty() {
            return Ok(());
        }
        // error logging is handled by run
        if let Err(err) = run(int, &input) {
            eprintln!("{err}");
        }
    }
}

fn run_file(int: &mut Interpreter, file_name: &str) -> MainResult {
    let mut file = File::open(file_name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    if let Err(err) = run(int, &contents) {
        eprintln!("{err}");
        std::process::exit(75);
    }
    Ok(())
}

fn main() -> MainResult {
    let args: Vec<String> = std::env::args().collect();
    let mut interpreter = Interpreter::default();
    match args.len() {
        1 => run_prompt(&mut interpreter),
        2 => run_file(&mut interpreter, &args[1]),
        _ => {
            eprintln!("usage: {} [script]", args[0]);
            std::process::exit(64);
        }
    }
}
