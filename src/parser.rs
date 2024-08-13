use crate::error::LoxError;
use crate::error::LoxResult;
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::Token;
use crate::token::TokenType;
use crate::token::TokenType::*;

type ResultExpr<'a> = LoxResult<Expr<'a>>;
type ResultStmt<'a> = LoxResult<Stmt<'a>>;

pub struct Parser<'long> {
    tokens: &'long [Token<'long>],
    current: usize,
}

impl<'long> Parser<'long> {
    pub fn new(tokens: &'long [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> LoxResult<Vec<Stmt>> {
        let mut statements = vec![];
        let mut errors = vec![];
        while !self.is_at_end() {
            match self.statement() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    errors.push(err);
                    self.synchronize();
                }
            }
        }
        if !errors.is_empty() {
            Err(LoxError::MultiError(errors))
        } else {
            Ok(statements)
        }
    }

    fn advance(&mut self) {
        self.current += 1
    }

    fn is_at_end(&self) -> bool {
        self.peek() == Eof
    }

    fn previous(&self) -> Token<'long> {
        self.tokens[self.current - 1]
    }

    fn peek(&self) -> TokenType {
        if self.current >= self.tokens.len() {
            Eof
        } else {
            self.tokens[self.current].token
        }
    }

    fn consume(&mut self, token_type: TokenType, err_msg: &str) -> LoxResult<()> {
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
            self.peek() == token_type
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

    fn statement(&mut self) -> ResultStmt<'long> {
        if self.token_match(&[Print]) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> ResultStmt<'long> {
        let expr = self.expression()?;
        self.consume(Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(expr))
    }

    fn expression_statement(&mut self) -> ResultStmt<'long> {
        let expr = self.expression()?;
        self.consume(Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Expr(expr))
    }

    pub fn expression(&mut self) -> ResultExpr<'long> {
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
            return Err(LoxError::UnexpectedEof);
        }
        let cur_token = self.peek();
        match cur_token {
            False | True | Nil | TNumber(_) | TString(_) => {
                let expr = Expr::Literal(cur_token.into());
                self.advance();
                Ok(expr)
            }
            LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(RightParen, "Expect ')' after expression")?;
                Ok(Expr::Grouping(Box::new(expr)))
            }
            _ => panic!("unreachable arm {cur_token:?}"),
        }
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token == Semicolon {
                return;
            }
            match self.peek() {
                Class | Fun | Var | For | If | While | Print | Return => return,
                _ => self.advance(),
            }
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
        "(== seven (/ (group (- (- 30) (/ 140 2))) (- 10)))"
    )]
    #[case("1 < 2 == 4 >= 3", "(== (< 1 2) (>= 4 3))")]
    fn test_parse_expr(#[case] input: &str, #[case] want: &str) -> Result<(), LoxError> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let expr = parser.expression()?;
        let got = format!("{}", expr);
        assert_eq!(got, want);
        Ok(())
    }

    #[rstest::rstest]
    #[case("4 + 5;", "expr((+ 4 5))")]
    #[case("print \"hello, world\";", "print(hello, world)")]
    fn test_parse_stmt(#[case] input: &str, #[case] want: &str) -> Result<(), LoxError> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let stmt = parser.statement()?;
        let got = format!("{}", stmt);
        assert_eq!(got, want);
        Ok(())
    }

    #[rstest::rstest]
    #[case("print 4;", "print(4)")]
    #[case("print nil;\ntrue;", "print(nil)\nexpr(true)")]
    fn test_parse(#[case] input: &str, #[case] want: &str) -> Result<(), LoxError> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let stmts = parser.parse()?;
        let mut got = String::new();
        for (i, stmt) in stmts.into_iter().enumerate() {
            if i > 0 {
                got.push('\n')
            }
            got.push_str(&format!("{stmt}"));
        }
        assert_eq!(got, want);
        Ok(())
    }
}
