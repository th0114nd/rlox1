use crate::error::LoxError;
use crate::error::ScanError;
use crate::models::Token;
use crate::models::TokenType;
use crate::models::TokenType::*;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::iter::Peekable;
use std::mem;
use std::str::CharIndices;

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenType> = HashMap::from_iter(
        vec![
            ("and", And),
            ("class", Class),
            ("else", Else),
            ("false", False),
            ("for", For),
            ("fun", Fun),
            ("if", If),
            ("nil", Nil),
            ("or", Or),
            ("print", Print),
            ("return", Return),
            ("super", Super),
            ("this", This),
            ("true", True),
            ("var", Var),
            ("while", While),
        ]
        .into_iter()
    );
}

pub struct Scanner<'a> {
    src: &'a str,
    chars: Peekable<CharIndices<'a>>,
    tokens: Vec<Token>,
    errors: Vec<ScanError>,

    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            chars: src.char_indices().peekable(),
            tokens: vec![],
            errors: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, LoxError> {
        while let Some((start, c)) = self.chars.peek() {
            self.start = *start;
            let c = *c;
            self.advance();
            self.scan_token(c);
        }
        self.tokens.push(Token {
            token: Eof,
            lexeme: "".into(),
            line: self.line,
        });
        if self.errors.is_empty() {
            Ok(mem::take(&mut self.tokens))
        } else {
            Err(mem::take(&mut self.errors))?
        }
    }

    fn buffered_str(&self) -> &'a str {
        &self.src[self.start..self.current + 1]
    }

    fn is_at_end(&mut self) -> bool {
        self.chars.peek().is_none()
    }

    fn advance(&mut self) {
        self.chars
            .next()
            .map(|(cur, _)| {
                self.current = cur;
            })
            .expect("I think we'll always have more? maybe not");
    }

    fn scan_token(&mut self, c: char) {
        match c {
            '(' => self.add_token(LeftParen),
            ')' => self.add_token(RightParen),
            '{' => self.add_token(LeftBrace),
            '}' => self.add_token(RightBrace),
            ',' => self.add_token(Comma),
            '.' => self.add_token(Dot),
            '-' => self.add_token(Minus),
            '+' => self.add_token(Plus),
            ';' => self.add_token(Semicolon),
            '*' => self.add_token(Star),
            '!' => {
                let has_match = self.match_char('=');
                self.add_token(if has_match { BangEqual } else { Bang });
            }
            '=' => {
                let has_match = self.match_char('=');
                self.add_token(if has_match { EqualEqual } else { Equal });
            }
            '<' => {
                let has_match = self.match_char('=');
                self.add_token(if has_match { LessEqual } else { Less });
            }
            '>' => {
                let has_match = self.match_char('=');
                self.add_token(if has_match { GreaterEqual } else { Greater });
            }
            '/' => {
                let has_match = self.match_char('/');
                if has_match {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(Slash);
                }
            }
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string(),
            d if d.is_ascii_digit() => self.number(),
            d if d.is_ascii_alphabetic() || d == '_' => self.identifier(),
            _ => self.add_error(format!("Unexpected character: {c}")),
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        match self.chars.peek() {
            None => false,
            Some((_, actual)) => {
                if *actual == expected {
                    self.advance();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.tokens.push(Token {
            token: token_type,
            line: self.line,
            lexeme: (&self.src[self.start..self.current + 1]).into(),
        })
    }

    fn add_error(&mut self, msg: std::string::String) {
        self.errors.push(ScanError {
            line: self.line,
            msg: msg.into(),
        })
    }

    fn peek(&mut self) -> char {
        match self.chars.peek() {
            None => '\0',
            Some((_, c)) => *c,
        }
    }

    fn peek_next(&mut self) -> char {
        // We need this specifically to check for a digit, so
        // ok to fail most code points.
        let b = self.src.as_bytes()[self.current + 2];
        char::from_u32(b as u32).unwrap_or('\0')
    }

    fn string(&mut self) {
        while let Some((_, c)) = self.chars.peek()
            && *c != '"'
        {
            if *c == '\n' {
                self.line += 1
            }
            self.advance();
        }
        if self.is_at_end() {
            self.add_error("Unterminated string.".to_owned());
            return;
        }
        self.advance();
        let value = self.buffered_str();
        self.add_token(TString((&value[1..value.len() - 1]).into()));
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume the "."
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        let num: f64 = self
            .buffered_str()
            .parse()
            .expect("this is already a number");
        self.add_token(TNumber(num));
    }

    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        let ident = self.buffered_str();
        self.add_token(match KEYWORDS.get(&ident) {
            Some(token_type) => token_type.clone(),
            None => Identifier,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::LoxError;

    #[rstest::rstest]
    #[case(
        "var language = \"lox\";\nvar pi = 3.14159;",
        vec![Var, Identifier, Equal, TString("lox".into()), Semicolon, Var, Identifier, Equal, TNumber(3.14159), Semicolon, Eof],
        vec!["var", "language", "=", "\"lox\"", ";", "var", "pi", "=", "3.14159", ";", ""],
    )]
    #[case(
        "(!= !{ -) + ==}=; / > >= < <= *",
        vec![
            LeftParen, BangEqual, Bang, LeftBrace, Minus, RightParen,
            Plus, EqualEqual, RightBrace, Equal, Semicolon, Slash, Greater,
            GreaterEqual, Less, LessEqual, Star, Eof
        ],
vec![ "(", "!=", "!", "{", "-", ")", "+", "==", "}", "=", ";", "/", ">", ">=", "<", "<=", "*", ""], )]
    #[case("and class else false for trap fun if nil or print return super this true var while",
        vec![
        And, Class, Else, False, For, Identifier, Fun, If, Nil, Or, Print, Return,
        Super, This, True, Var, While, Eof,
    ], vec![
        "and", "class", "else", "false", "for", "trap", "fun", "if", "nil",
        "or", "print", "return", "super", "this", "true", "var", "while", "",
    ])]
    #[case("  var \t  x   = // a comment doesn't stop this\n 1894",
        vec![Var, Identifier, Equal, TNumber(1894.0), Eof],
        vec!["var", "x", "=", "1894", ""],
    )]
    fn test_scan_types(
        #[case] input: &str,
        #[case] want_types: Vec<TokenType>,
        #[case] want_lexemes: Vec<&str>,
    ) -> Result<(), LoxError> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;

        let got_types: Vec<_> = tokens.iter().map(|token| token.token.clone()).collect();
        let got_lexemes: Vec<_> = tokens.iter().map(|token| token.lexeme.clone()).collect();
        assert_eq!(got_types, want_types);
        assert_eq!(got_lexemes, want_lexemes);
        Ok(())
    }

    #[rstest::rstest]
    #[case(
        "var x = \"interrupted string ends here",
        "[line 1] Error: Unterminated string."
    )]
    #[case("\n\n#nofilter", "[line 3] Error: Unexpected character: #")]
    fn test_scan_types_error(#[case] input: &str, #[case] want: &str) {
        let mut scanner = Scanner::new(input);
        let err = scanner.scan_tokens().expect_err("should fail to scan");
        assert_eq!(format!("{}", LoxError::from(err)), want,);
    }
}
