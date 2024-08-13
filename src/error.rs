use crate::token::Token;
use crate::token::TokenType;
use crate::value;
use std::fmt::Display;
use std::fmt::Write;
use std::io;
use thiserror::Error;

pub type LoxResult<T> = Result<T, LoxError>;

#[derive(Debug, Error)]
pub enum LoxError {
    #[error("[line {0}] Error: {1}")]
    ScanError(usize, String),

    #[error("{0}")]
    ParseError(String),
    #[error("unexpected eof")]
    UnexpectedEof,

    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("{}", join_all(.0))]
    MultiError(Vec<LoxError>),

    #[error("value error: {0}")]
    ValueError(#[from] value::ValueError),
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
