use crate::models::Token;
use crate::models::TokenType;
use crate::models::Value;
use crate::resolver::ResolverError;
use compact_str::CompactString;
use std::fmt::Display;
use std::fmt::Write;
use std::io;
use thiserror::Error;

pub type LoxResult<T> = Result<T, LoxError>;

#[derive(Debug, Error, PartialEq)]
#[error("[line {line}] Error: {msg}")]
pub struct ScanError {
    pub line: usize,
    pub msg: CompactString,
}

#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    #[error("[line {0}] {1}")]
    GeneralError(usize, CompactString),
    #[error("[line {0}] Error: unexpected eof")]
    UnexpectedEof(usize),
}

// TODO: I think its probably still correct that there should be two levels to the hierarchy:
// ValueErrors are unlabeled and the result of a computation
// RuntimeErrors are the result of a statement evaluation and can provide a line number
// So we should have RuntimeError(line, ValueError)

#[derive(Debug, Error, PartialEq)]
pub enum RuntimeError {
    #[error("[line {line}] Error: arity mismatch {got} vs {want}")]
    ArityMismatch {
        line: CompactString,
        got: usize,
        want: usize,
    },

    #[error("[line {line}] Error: type mismatch: {lhs} vs {rhs}")]
    TypeMismatch {
        line: CompactString,
        lhs: Value,
        rhs: Value,
    },

    #[error("[line {line}] Error: division by zero")]
    ZeroDivError { line: CompactString },

    #[error("[line {line}] Error: system time error")]
    SystemTimeError { line: CompactString },

    #[error("[line {line}] Error: undefined variable: '{name}'")]
    UndefinedVariable {
        line: CompactString,
        name: CompactString,
    },

    #[error("[line {line}]: Error: undefined property: '{name}'")]
    UndefinedProperty {
        line: CompactString,
        name: CompactString,
    },

    #[error("[line {line} Error: non callable called {value}")]
    NonCallableCalled { line: CompactString, value: Value },

    #[error("[line {line}] return {value}(not an error!)")]
    Return { line: CompactString, value: Value },

    #[error("[line {line}] break (not an error!)")]
    Break { line: CompactString },
}

#[derive(Debug, Error, PartialEq)]
pub enum LoxError {
    #[error("{}", join_all(.0))]
    ScanErrors(Vec<ScanError>),

    #[error("{}", join_all(.0))]
    ParseErrors(Vec<ParseError>),

    #[error("{}", join_all(.0))]
    ResolverErrors(Vec<ResolverError>),

    #[error("{0}")]
    RuntimeError(#[from] RuntimeError),
}

#[derive(Debug, Error)]
pub enum MainError {
    #[error("io error: {0}")]
    IoError(#[from] io::Error),

    #[error(transparent)]
    LoxError(#[from] LoxError),
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

impl<S: AsRef<str>> From<(Token, S)> for ParseError {
    fn from((token, msg): (Token, S)) -> ParseError {
        let r#where = if token.token == TokenType::Eof {
            "end".to_owned()
        } else {
            format!("'{}'", token.lexeme)
        };
        ParseError::GeneralError(
            token.line,
            format!("Error at {}: {}", r#where, msg.as_ref()).into(),
        )
    }
}

impl From<Vec<ScanError>> for LoxError {
    fn from(vec_errs: Vec<ScanError>) -> LoxError {
        LoxError::ScanErrors(vec_errs)
    }
}

impl From<Vec<ParseError>> for LoxError {
    fn from(vec_errs: Vec<ParseError>) -> LoxError {
        LoxError::ParseErrors(vec_errs)
    }
}

impl From<Vec<ResolverError>> for LoxError {
    fn from(vec_errs: Vec<ResolverError>) -> LoxError {
        LoxError::ResolverErrors(vec_errs)
    }
}
