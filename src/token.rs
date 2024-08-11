use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType<'a> {
    // single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String(&'a str),
    Number(f64),

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Token<'a> {
    pub token: TokenType<'a>,
    pub lexeme: &'a str,
    pub line: usize,
}

impl<'a> Display for Token<'a> {
    fn fmt(self: &Token<'a>, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {:?}", self.token, self.lexeme)
    }
}
