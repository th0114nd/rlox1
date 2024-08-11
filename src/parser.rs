use crate::error;
use crate::expr::Expr;
use crate::token::Token;
use crate::token::TokenType;
use crate::token::TokenType::*;

pub struct Parser<'long> {
    tokens: &'long [Token<'long>],
    current: usize,
}

impl<'long, 'short> Parser<'long> {
    fn new(tokens: &'long [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    fn advance(&mut self) {
        self.current += 1
    }

    fn is_at_end(&self) -> bool {
        self.peek().token == Eof
    }

    fn previous(&'short self) -> Token<'long> {
        self.tokens[self.current - 1]
    }

    fn peek(&'short self) -> Token<'long> {
        self.tokens[self.current]
    }

    fn consume(&mut self, token_type: TokenType, err_msg: &str) {
        let token = self.tokens[self.current];
        if token.token != token_type {
            error(token.line, err_msg)
        }
        self.advance();
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token == token_type
        }
    }

    fn token_match(&'short mut self, types: &'static [TokenType]) -> bool {
        for token_type in types {
            if self.check(*token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn expression(&'short mut self) -> Expr<'long> {
        self.equality()
    }

    fn bin_op(
        &'short mut self,
        token_types: &'static [TokenType],
        next_op: impl Fn(&mut Self) -> Expr<'long>,
        //next_op: F,
    ) -> Expr<'long>
//where
    //    F: Fn(&mut Self) -> Expr<'long>, //where
                                         //'long: 'short,
    {
        let mut expr = next_op(self);

        while self.token_match(token_types) {
            let operator = self.previous();
            let right = Box::new(next_op(self));
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            }
        }
        expr
    }

    fn equality(&'short mut self) -> Expr<'long> {
        let mut expr = self.comparison();

        while self.token_match(&[BangEqual, EqualEqual]) {
            let operator = self.previous();
            let right = Box::new(self.comparison());
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        expr
    }

    fn comparison(&'short mut self) -> Expr<'long> {
        let mut expr = self.term();

        while self.token_match(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous();
            let right = Box::new(self.term());
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            }
        }
        expr
    }

    fn term(&'short mut self) -> Expr<'long> {
        let mut expr = self.factor();

        while self.token_match(&[Plus, Minus]) {
            let operator = self.previous();
            let right = Box::new(self.factor());
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            }
        }
        expr
    }

    fn factor(&'short mut self) -> Expr<'long> {
        let mut expr = self.unary();

        while self.token_match(&[Star, Slash]) {
            let operator = self.previous();
            let right = Box::new(self.unary());
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            }
        }
        expr
    }

    fn unary(&'short mut self) -> Expr<'long> {
        if self.token_match(&[Bang, Minus]) {
            let operator = self.previous();
            let right = Box::new(self.unary());
            Expr::Unary { operator, right }
        } else {
            self.primary()
        }
    }

    fn primary(&'short mut self) -> Expr<'long> {
        if self.is_at_end() {
            panic!("I shouldn't be at the end")
        }
        let cur_token = self.peek();
        match cur_token.token {
            False | True | Nil | Number(_) | String(_) => {
                let expr = Expr::Literal(cur_token);
                self.advance();
                expr
            }
            LeftParen => {
                self.advance();
                let expr = Box::new(self.expression());
                self.consume(RightParen, "Expect ')' after expression");
                Expr::Grouping(expr)
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
    fn test_parse_expr(#[case] input: &str, #[case] want: &str) {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(&tokens);
        let expr = parser.expression();
        let got = format!("{}", expr);
        assert_eq!(got, want);
    }
}
