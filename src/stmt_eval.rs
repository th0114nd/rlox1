use crate::stmt::Stmt;
use crate::value::ValueError;
use std::io;

impl<'a> Stmt<'a> {
    pub fn eval(&self, mut w: impl io::Write) -> Result<(), ValueError> {
        match self {
            Stmt::Expr(expr) => {
                expr.eval()?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let v = expr.eval()?;
                writeln!(w, "{v}").expect("writes should not fail");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::LoxResult;

    #[rstest::rstest]
    #[case("nil;", vec![()], "")]
    #[case("print nil;", vec![()], "nil\n")]
    #[case("print nil;\ntrue;", vec![(), ()], "nil\n")]
    #[case("true;", vec![()], "")]
    #[case("print 3 + 4; 10;", vec![(), ()], "7\n")]
    #[case("print 3 + 4; print \"hello\";", vec![(), ()], "7\nhello\n")]
    fn test_eval(
        #[case] input: &str,
        #[case] want: Vec<()>,
        #[case] want_stdout: &'static str,
    ) -> LoxResult<()> {
        let mut scanner = crate::scanner::Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = crate::parser::Parser::new(&tokens);
        let stmts = parser.parse()?;

        let mut buf = vec![];

        let got: Vec<()> = stmts
            .into_iter()
            .map(|ref s| s.eval(&mut buf))
            .collect::<Result<Vec<()>, ValueError>>()?;
        assert_eq!(got, want);

        assert_eq!(std::str::from_utf8(&buf), Ok(want_stdout));
        Ok(())
    }
}
