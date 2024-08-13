use crate::error::LoxError;
use crate::stmt::Stmt;
use std::io;

impl<'a> Stmt<'a> {
    pub fn eval(&self, current: usize, mut w: impl io::Write) -> Result<(), LoxError> {
        match self {
            Stmt::Expr(expr) => {
                expr.eval(current)?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let v = expr.eval(current)?;
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
    use crate::parser::Parser;
    use crate::scanner::Scanner;

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
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let stmts = parser.parse()?;

        let mut buf = vec![];

        let got: Vec<()> = stmts
            .into_iter()
            .enumerate()
            .map(|(current, ref s)| s.eval(current, &mut buf))
            .collect::<Result<Vec<()>, LoxError>>()?;
        assert_eq!(got, want);

        assert_eq!(std::str::from_utf8(&buf), Ok(want_stdout));
        Ok(())
    }

    #[rstest::rstest]
    #[case(
        "print nil;\n 4 + \"lox\";\n 2 + \"oops\";",
        "[line 2] Error: value error: type mismatch: 4 vs lox",
        "nil\n"
    )]
    fn test_eval_error(
        #[case] input: &str,
        #[case] want: &str,
        #[case] want_stdout: &str,
    ) -> LoxResult<()> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let stmts = parser.parse()?;

        let mut buf = vec![];

        let got = stmts
            .into_iter()
            .enumerate()
            .map(|(current, ref s)| s.eval(current + 1, &mut buf))
            .collect::<Result<Vec<()>, LoxError>>()
            .expect_err("should have created an error");

        assert_eq!(format!("{got}"), want);

        assert_eq!(std::str::from_utf8(&buf), Ok(want_stdout));
        Ok(())
    }
}
