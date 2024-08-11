#![feature(let_chains)]
use std::fs::File;
use std::io::stdin;
use std::io::BufRead;
use std::io::Write;
use std::io::{self, Read};

mod error;
mod expr;
mod parser;
mod scanner;
mod token;

use crate::scanner::Scanner;
//use crate::token::Token;
//use crate::token::TokenType;

//lazy_static! {
//    static ref HAS_FAILURE: Mutex<bool> = Mutex::new(false);
//}

//trait Reportable {
//    fn rep(self) -> String;
//}
//
//impl Reportable for usize {
//    fn line(self) -> String {
//        format!("[line {line}] Error:")
//    }
//}
//
//impl<'a> Reportable for Token<'a> {
//    fn rep(self) -> String {
//        let r#where = if self.token == TokenType::Eof {
//            "at end".to_owned()
//        } else {
//            format!("at '{}'", self.lexeme)
//        };
//        let line = self.line;
//        format!("[line {line}] Error {where}: ")
//    }
//}

//fn report(line: usize, r#where: impl AsRef<str>, msg: impl AsRef<str>) {
//    println!("[line {line}] Error{}: {}", r#where.as_ref(), msg.as_ref());
//    set_error();
//}
//
//fn set_error() {
//    *HAS_FAILURE.lock().unwrap() = true;
//}
//
//fn reset_error() {
//    *HAS_FAILURE.lock().unwrap() = false;
//}

//fn error(token: Token, msg: impl AsRef<str>) {
//    if token.token == TokenType::Eof {
//        report(token.line, " at end", msg);
//    } else {
//        report(token.line, " at '".to_owned() + token.lexeme + "'", msg);
//    }
//}

//fn run(src: String) -> Result<(), Error> {}
fn run(src: &str) -> Result<(), error::LoxError> {
    let mut scanner = Scanner::new(src);
    let maybe_tokens = scanner.scan_tokens();
    if let Err(errs) = maybe_tokens {
        for err in &errs {
            eprintln!("{err}");
        }
        return Err(error::LoxError::from(errs));
    }
    let tokens = maybe_tokens.unwrap();
    for token in tokens {
        println!("{token}");
    }
    Ok(())
}

fn run_prompt() -> Result<(), error::LoxError> {
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
        //error::reset_error();
    }
}

fn run_file(file_name: &str) -> Result<(), error::LoxError> {
    let mut file = File::open(file_name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    run(&contents)
}

fn main() -> Result<(), error::LoxError> {
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
