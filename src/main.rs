#![feature(let_chains)]
use std::fs::File;
use std::io::stdin;
use std::io::BufRead;
use std::io::Write;
use std::io::{self, Read};

mod error;
mod expr;
mod expr_eval;
mod parser;
mod scanner;
mod stmt;
mod stmt_eval;
mod token;
mod value;

use crate::error::LoxError;
use crate::error::LoxResult;
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::value::ValueError;

fn run(src: &str) -> LoxResult<()> {
    let mut scanner = Scanner::new(src);
    let maybe_tokens = scanner.scan_tokens();
    if let Err(errs) = maybe_tokens {
        for err in &errs {
            eprintln!("{err}");
        }
        return Err(LoxError::from(errs));
    }
    let tokens = maybe_tokens.unwrap();

    let mut parser = Parser::new(&tokens);
    let stmts = parser.parse();
    match stmts {
        Err(err) => eprintln!("{err}"),
        Ok(stmts) => stmts.iter().try_for_each(|s| s.eval(io::stdout()))?,
    }
    Ok(())
}

fn run_prompt() -> LoxResult<()> {
    let mut input = String::new();
    loop {
        input.clear();
        print!("> ");
        io::stdout().flush()?;
        stdin().lock().read_line(&mut input)?;
        if input.is_empty() {
            return Ok(());
        }
        // TODO: log?
        let _ = run(&input);
    }
}

fn run_file(file_name: &str) -> LoxResult<()> {
    let mut file = File::open(file_name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    run(&contents)
}

fn main() -> LoxResult<()> {
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
