use crate::error;
use crate::error::LoxError;
use crate::expr::Expr;
use crate::token::Token;
use crate::token::TokenType;
use crate::token::TokenType::*;
use thiserror::Error;

pub struct Parser<'long> {
    tokens: &'long [Token<'long>],
    current: usize,
}

//#[derive(Debug, Error)]
//#[error("parser error")]
//struct ParseError {}
//
//impl ParseError {
//    fn new(token: Token, msg: impl AsRef<str>) -> Self {
//        error(token, msg);
//        ParseError {}
//    }
//}

type ParseResult<T> = Result<T, LoxError>;
type ResultExpr<'a> = ParseResult<Expr<'a>>;

impl<'long> Parser<'long> {
    fn new(tokens: &'long [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    fn advance(&mut self) {
        self.current += 1
    }

    fn is_at_end(&self) -> bool {
        self.peek().token == Eof
    }

    fn previous(&self) -> Token<'long> {
        self.tokens[self.current - 1]
    }

    fn peek(&self) -> Token<'long> {
        self.tokens[self.current]
    }

    fn consume(&mut self, token_type: TokenType, err_msg: &str) -> ParseResult<()> {
        let token = self.tokens[self.current];
        if token.token != token_type {
            return Err(LoxError::from((token, err_msg)));
        }
        self.advance();
        Ok(())
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token == token_type
        }
    }

    fn token_match(&mut self, types: &'static [TokenType]) -> bool {
        for token_type in types {
            if self.check(*token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn expression(&mut self) -> ParseResult<Expr<'long>> {
        self.equality()
    }

    fn bin_op(
        &mut self,
        token_types: &'static [TokenType],
        mut next_op: impl FnMut(&mut Self) -> ResultExpr<'long>,
    ) -> ResultExpr<'long> {
        let mut expr = next_op(self)?;

        while self.token_match(token_types) {
            let operator = self.previous();
            let right = next_op(self)?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn equality(&mut self) -> ResultExpr<'long> {
        self.bin_op(&[BangEqual, EqualEqual], |s| s.comparison())
    }

    fn comparison(&mut self) -> ResultExpr<'long> {
        self.bin_op(&[Greater, GreaterEqual, Less, LessEqual], |s| s.term())
    }

    fn term(&mut self) -> ResultExpr<'long> {
        self.bin_op(&[Plus, Minus], |s| s.factor())
    }

    fn factor(&mut self) -> ResultExpr<'long> {
        self.bin_op(&[Star, Slash], |s| s.unary())
    }

    fn unary(&mut self) -> ResultExpr<'long> {
        if self.token_match(&[Bang, Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> ResultExpr<'long> {
        if self.is_at_end() {
            panic!("I shouldn't be at the end")
        }
        let cur_token = self.peek();
        match cur_token.token {
            False | True | Nil | Number(_) | String(_) => {
                let expr = Expr::Literal(cur_token);
                self.advance();
                Ok(expr)
            }
            LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(RightParen, "Expect ')' after expression")?;
                Ok(Expr::Grouping(Box::new(expr)))
            }
            _ => panic!("unreachable arm {cur_token}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Scanner;

    #[rstest::rstest]
    #[case("6", "6")]
    #[case("6 / 3 - 1", "(- (/ 6 3) 1)")]
    #[case("1 + 2 + 3", "(+ (+ 1 2) 3)")]
    #[case("10 / 2 / 1", "(/ (/ 10 2) 1)")]
    #[case(
        "\"seven\" == (-30 - 140 / 2) / -10",
        "(== \"seven\" (/ (group (- (- 30) (/ 140 2))) (- 10)))"
    )]
    #[case("1 < 2 == 4 >= 3", "(== (< 1 2) (>= 4 3))")]
    fn test_parse_expr(#[case] input: &str, #[case] want: &str) -> Result<(), LoxError> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let expr = parser.expression().expect("should have been able to parse");
        let got = format!("{}", expr);
        assert_eq!(got, want);
        Ok(())
    }
}
