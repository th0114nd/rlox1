#![feature(let_chains)]
use std::fs::File;
use std::io;
use std::io::stdin;
use std::io::BufRead;
use std::io::Read;
use std::io::Write;

mod environment;
mod error;
mod expr;
mod expr_eval;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
mod stmt_eval;
mod token;
mod value;

use crate::error::LoxResult;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::scanner::Scanner;

fn run(int: &mut Interpreter<io::Stdout>, src: &str) -> LoxResult<()> {
    let mut scanner = Scanner::new(src);
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(&tokens);
    let stmts = parser.parse()?;

    int.interpret(stmts).map(|_| ())
}

fn run_prompt(int: &mut Interpreter<io::Stdout>) -> LoxResult<()> {
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

fn run_file(int: &mut Interpreter<io::Stdout>, file_name: &str) -> LoxResult<()> {
    let mut file = File::open(file_name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    if let Err(err) = run(int, &contents) {
        eprintln!("{err}");
        std::process::exit(75);
    }
    Ok(())
}

fn main() -> LoxResult<()> {
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
