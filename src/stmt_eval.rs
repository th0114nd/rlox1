use crate::stmt::Stmt;
use crate::value::Value;
use crate::value::ValueError;
use std::io;
use std::str;

impl<'a> Stmt<'a> {
    fn eval(&self, mut w: impl io::Write) -> Result<Value, ValueError> {
        match self {
            Stmt::Expr(expr) => {
                expr.eval()?;
                Ok(Value::VNil)
            }
            Stmt::Print(expr) => {
                let v = expr.eval()?;
                writeln!(w, "{v}").expect("writes should not fail");
                Ok(Value::VNil)
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
    #[case("nil;", vec![VNil], "")]
    #[case("print nil;", vec![VNil], "nil\n")]
    #[case("print nil;\ntrue;", vec![VNil, VNil], "nil\n")]
    #[case("true;", vec![VNil], "")]
    #[case("print 3 + 4; 10;", vec![VNil, VNil], "7\n")]
    #[case("print 3 + 4; print \"hello\";", vec![VNil, VNil], "7\nhello\n")]
    fn test_eval(
        #[case] input: &str,
        #[case] want: Vec<Value>,
        #[case] want_stdout: &'static str,
    ) -> LoxResult<()> {
        let mut scanner = crate::scanner::Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = crate::parser::Parser::new(&tokens);
        let stmts = parser.parse()?;

        let mut buf = vec![];

        let got: Vec<Value> = stmts
            .into_iter()
            .map(|ref s| s.eval(&mut buf))
            .collect::<Result<Vec<Value>, ValueError>>()?;
        assert_eq!(got, want);

        assert_eq!(str::from_utf8(&buf), Ok(want_stdout));
        Ok(())
    }
}
