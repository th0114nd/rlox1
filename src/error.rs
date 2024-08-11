use crate::token::Token;
use crate::token::TokenType;
use lazy_static::lazy_static;
use std::fmt::Display;
use std::fmt::Write;
use std::io;
use std::sync::Mutex;
use thiserror::Error;

lazy_static! {
    static ref HAS_FAILURE: Mutex<bool> = Mutex::new(false);
}

fn report(line: usize, r#where: impl AsRef<str>, msg: impl AsRef<str>) {
    println!("[line {line}] Error{}: {}", r#where.as_ref(), msg.as_ref());
    set_error();
}

pub fn set_error() {
    *HAS_FAILURE.lock().unwrap() = true;
}

pub fn reset_error() {
    *HAS_FAILURE.lock().unwrap() = false;
}

#[derive(Debug, Error)]
pub enum LoxError {
    #[error("[line {0}] Error: {1}")]
    ScanError(usize, String),
    #[error("{0}")]
    ParseError(String),

    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("{}", join_all(.0))]
    MultiError(Vec<LoxError>),
}

fn join_all<T: Display>(items: &[T]) -> String {
    let mut out = String::new();
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push('\n')
        }
        write!(&mut out, "{}", item).expect("must be able to append to string");
    }
    out
}

impl<'a, T: AsRef<str>> From<(Token<'a>, T)> for LoxError {
    fn from((token, msg): (Token<'a>, T)) -> LoxError {
        let r#where = if token.token == TokenType::Eof {
            "end".to_owned()
        } else {
            format!("'{}'", token.lexeme)
        };
        LoxError::ParseError(format!(
            "[line {}] Error at {}: {}",
            token.line,
            r#where,
            msg.as_ref(),
        ))
    }
}

impl From<Vec<LoxError>> for LoxError {
    fn from(vec: Vec<LoxError>) -> LoxError {
        LoxError::MultiError(vec)
    }
}
