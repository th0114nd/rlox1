use compact_str::CompactString;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
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
    TString(CompactString),
    TNumber(f64),

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

// if this doesn't include the current offset in the line, can these refer to different tokens?
// we should be able to ignore
// or what about &Token or *Token, what if they are copied?

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token: TokenType,
    pub lexeme: CompactString,
    pub line: usize,
}

impl Display for Token {
    fn fmt(self: &Token, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {:?}", self.token, self.lexeme)
    }
}
