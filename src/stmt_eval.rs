use crate::expr::Expr;
use crate::token::TokenType::*;
use crate::value::Value;
use crate::value::ValueError;

impl<'a> Stmt<'a> {
    fn eval(&self) -> Result<Value, ValueError> {
        match self {
            Stmt::Expr(expr) => expr.eval(),
            Stmt::Print(expr) => {
                let v = expr.eval();
                println!("{v}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::LoxResult;
    use Value::*;

    #[rstest::rstest]
    #[case("nil", VNil)]
    #[case("true", Bool(true))]
    fn test_eval(#[case] input: &str, #[case] want: Value) -> LoxResult<()> {
        let mut scanner = crate::scanner::Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = crate::parser::Parser::new(&tokens);
        let stmts = parser.parse()?;
        for stmt in stmts {
            let got = stmt.eval()?;
            assert_eq!(got, want)
        }
        Ok(())
    }
}
