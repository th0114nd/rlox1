use crate::error::LoxError;
use crate::error::LoxResult;
use models::Expr;
use models::Stmt;
use models::StmtList;
use models::Token;
use models::TokenType;
use models::TokenType::*;
use models::Value;
use std::mem;

type ResultExpr<'a> = LoxResult<Expr<'a>>;
type ResultStmt<'a> = LoxResult<Stmt<'a>>;

pub struct Parser<'long> {
    tokens: &'long [Token<'long>],
    current: usize,
    statements: Vec<Stmt<'long>>,
    errors: Vec<LoxError>,
}

impl<'long> Parser<'long> {
    pub fn new(tokens: &'long [Token]) -> Self {
        Self {
            tokens,
            current: 0,
            statements: vec![],
            errors: vec![],
        }
    }

    pub fn parse(&mut self) -> LoxResult<StmtList> {
        while !self.is_at_end() {
            self.line();
        }
        if !self.errors.is_empty() {
            Err(LoxError::MultiError(mem::take(&mut self.errors)))
        } else {
            Ok(StmtList(mem::take(&mut self.statements)))
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

    fn line(&mut self) {
        match self.declaration() {
            Err(err) => {
                self.errors.push(err);
                self.synchronize();
            }
            Ok(stmt) => self.statements.push(stmt),
        };
    }

    fn declaration(&mut self) -> ResultStmt<'long> {
        if self.token_match(&[Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }
    fn var_declaration(&mut self) -> ResultStmt<'long> {
        self.consume(Identifier, "expected identifier in declaration")?;
        let lhs = self.previous();
        let rhs: Option<Expr<'long>> = if self.token_match(&[Equal]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(Semicolon, "Expected ';' after variable declaration")?;
        let line = self.previous().line;
        Ok(Stmt::VarDecl(line, lhs, rhs))
    }

    fn statement(&mut self) -> ResultStmt<'long> {
        if self.token_match(&[For]) {
            return self.for_statement();
        }
        if self.token_match(&[While]) {
            return self.while_statement();
        }
        if self.token_match(&[Print]) {
            return self.print_statement();
        }
        if self.token_match(&[If]) {
            return self.if_statement();
        }
        if self.token_match(&[LeftBrace]) {
            return self.block();
        }
        self.expression_statement()
    }

    fn for_statement(&mut self) -> ResultStmt<'long> {
        self.consume(LeftParen, "Expect '(' around condition")?;
        let line = self.previous().line;
        let init_stmt = if self.token_match(&[Semicolon]) {
            None
        } else if self.token_match(&[Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };
        let end_expr = if self.token_match(&[Semicolon]) {
            Expr::Literal(Value::Bool(true))
        } else {
            let expr = self.expression()?;
            self.consume(Semicolon, "Expect ';' in for condition (end)")?;
            expr
        };
        let update_expr: Expr = if self.token_match(&[RightParen]) {
            Expr::Literal(Value::VNil)
        } else {
            let expr = self.expression()?;
            self.consume(RightParen, "Expect ')' in for condition (update)")?;
            expr
        };
        let body = self.statement()?;
        let while_stmt = Stmt::While(
            line,
            end_expr,
            Box::new(Stmt::Block(vec![body, Stmt::Expr(line, update_expr)])),
        );
        Ok(match init_stmt {
            None => while_stmt,
            Some(init_stmt) => Stmt::Block(vec![init_stmt, while_stmt]),
        })
    }

    fn while_statement(&mut self) -> ResultStmt<'long> {
        self.consume(LeftParen, "Expect '(' around condition")?;
        let expr = self.expression()?;
        self.consume(RightParen, "Expect ')' around condition")?;
        let stmt = Box::new(self.statement()?);
        let line = self.previous().line;
        Ok(Stmt::While(line, expr, stmt))
    }

    fn if_statement(&mut self) -> ResultStmt<'long> {
        self.consume(LeftParen, "Expect '(' around condition")?;
        let if_expr = self.expression()?;
        self.consume(RightParen, "Expect ')' around condition")?;
        let then_stmt = Box::new(self.declaration()?);

        let else_stmt = if self.token_match(&[Else]) {
            Some(Box::new(self.declaration()?))
        } else {
            None
        };
        let line = self.previous().line;
        Ok(Stmt::IfThenElse {
            line,
            if_expr,
            then_stmt,
            else_stmt,
        })
    }

    fn print_statement(&mut self) -> ResultStmt<'long> {
        let expr = self.expression()?;
        self.consume(Semicolon, "Expect ';' after value.")?;
        let line = self.previous().line;
        Ok(Stmt::Print(line, expr))
    }

    fn block(&mut self) -> ResultStmt<'long> {
        let mut statements = vec![];
        while !self.check(RightBrace) && !self.is_at_end() {
            let decl = self.declaration()?;
            statements.push(decl);
        }
        self.consume(RightBrace, "Expect '}' to end block.")?;
        Ok(Stmt::Block(statements))
    }

    fn expression_statement(&mut self) -> ResultStmt<'long> {
        let expr = self.expression()?;
        self.consume(Semicolon, "Expect ';' after value.")?;
        let line = self.previous().line;
        Ok(Stmt::Expr(line, expr))
    }

    pub fn expression(&mut self) -> ResultExpr<'long> {
        self.assignment()
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

    fn bin_logic_op(
        &mut self,
        token_types: &'static [TokenType],
        mut next_op: impl FnMut(&mut Self) -> ResultExpr<'long>,
    ) -> ResultExpr<'long> {
        let mut expr = next_op(self)?;

        while self.token_match(token_types) {
            let operator = self.previous();
            let right = next_op(self)?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn assignment(&mut self) -> ResultExpr<'long> {
        let expr = self.logic_or()?;
        if self.token_match(&[Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;
            match expr {
                Expr::Variable(name) => Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                }),
                _ => Err(LoxError::from((equals, "Invalid assignment target."))),
            }
        } else {
            Ok(expr)
        }
    }

    fn logic_or(&mut self) -> ResultExpr<'long> {
        self.bin_logic_op(&[Or], |s| s.logic_and())
    }

    fn logic_and(&mut self) -> ResultExpr<'long> {
        self.bin_logic_op(&[And], |s| s.equality())
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
        let cur_token = self.peek();
        match cur_token {
            Eof => Err(LoxError::UnexpectedEof(self.current)),
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
            Identifier => {
                self.advance();
                Ok(Expr::Variable(self.previous()))
            }
            _ => {
                let token = self.tokens[self.current];
                let err_msg = "unexpected token";
                Err(LoxError::from((token, err_msg)))
            }
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
    #[case("foo + bar - baz", "(- (+ v#foo v#bar) v#baz)")]
    #[case("a = 4", "(= v#a 4)")]
    #[case("a = b = \"hello\"", "(= v#a (= v#b hello))")]
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
    #[case("var x = 17 + 1;", "var(x = (+ 17 1))")]
    #[case("var y;", "var(y)")]
    fn test_parse_stmt(#[case] input: &str, #[case] want: &str) -> Result<(), LoxError> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let stmt = parser.declaration()?;
        let got = format!("{}", stmt);
        assert_eq!(got, want);
        Ok(())
    }

    #[rstest::rstest]
    #[case("print 4;", "print(4)\n")]
    #[case("print nil;\ntrue;", "print(nil)\nexpr(true)\n")]
    #[case("{}", "{\n}\n")]
    #[case("{ print nil; }", "{\nprint(nil)\n}\n")]
    #[case(
        "if (first) if (second) var x = 1; else 2;",
        "(if v#first (if v#second var(x = 1) expr(2)) {})\n"
    )]
    #[case(
        "if (\"hello\") { 1; } else { 2; }",
        "(if hello {\nexpr(1)\n} {\nexpr(2)\n})\n"
    )]
    #[case("true and false or 3 == 4;", "expr((or (and true false) (== 3 4)))\n")]
    #[case(
        "for (;;) {}",
        r#"(while true {
{
}
expr(nil)
})
"#
    )]
    #[case(
        "for (var i = 0; i < 10; i = i + 1) print i;",
        r#"{
var(i = 0)
(while (< v#i 10) {
print(v#i)
expr((= v#i (+ v#i 1)))
})
}
"#
    )]

    fn test_parse(#[case] input: &str, #[case] want: &str) -> Result<(), LoxError> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let stmts = parser.parse()?;
        let got = format!("{}", stmts);
        assert_eq!(got, want);
        Ok(())
    }

    #[rstest::rstest]
    #[case(
        "( \"partial\" + \"group\" ;",
        "[line 1] Error at ';': Expect ')' after expression"
    )]
    #[case("2 +", "[line 2] Error: unexpected eof")]
    #[case("+ 1", "[line 1] Error at '+': unexpected token")]
    #[case("2 + ;", "[line 1] Error at ';': unexpected token")]
    #[case("print 4\n2 + 4", "[line 2] Error at '2': Expect ';' after value.")]
    #[case("print 4;\n 2 + 4", "[line 2] Error at end: Expect ';' after value.")]
    #[case(
        "var 72;",
        "[line 1] Error at '72': expected identifier in declaration"
    )]
    #[case(
        "var 72 = 4;",
        "[line 1] Error at '72': expected identifier in declaration"
    )]
    #[case(
        "var ident + 2 = \"value\";",
        "[line 1] Error at '+': Expected ';' after variable declaration"
    )]
    #[case(
        "var y",
        "[line 1] Error at end: Expected ';' after variable declaration"
    )]
    #[case("17 = a", "[line 1] Error at '=': Invalid assignment target.")]
    #[case("a = 17 = b", "[line 1] Error at '=': Invalid assignment target.")]
    fn test_parse_errors(#[case] input: &str, #[case] want: &str) -> Result<(), LoxError> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let got = parser.parse().expect_err("should have failed to parse");
        assert_eq!(format!("{}", got), want);
        Ok(())
    }
}
