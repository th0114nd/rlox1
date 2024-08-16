use crate::environment::Env;
use crate::environment::Environment;
use crate::error::LoxError;
use crate::stmt::Stmt;
use crate::value::Value;
use std::io;

impl<'a> Stmt<'a> {
    pub fn eval(
        &self,
        current: usize,
        mut w: impl io::Write,
        env: &mut Environment,
    ) -> Result<(), LoxError> {
        match self {
            Stmt::Expr(expr) => {
                expr.eval(current, env)?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let v = expr.eval(current, env)?;
                writeln!(w, "{v}").expect("writes should not fail");
                Ok(())
            }
            Stmt::VarDecl(token, expr) => {
                let value = match expr {
                    None => Value::VNil,
                    Some(expr) => expr.eval(current, env)?,
                };
                env.define(token.lexeme, value);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::LoxResult;
    use crate::interpreter::Interpreter;
    use crate::parser::Parser;
    use crate::scanner::Scanner;

    fn str_eval(input: &str, buf: &mut Vec<u8>) -> LoxResult<Vec<()>> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let stmts = parser.parse()?;

        let mut interpreter = Interpreter {
            environment: Environment::default(),
            w: buf,
        };
        interpreter.interpret(stmts)
    }

    #[rstest::rstest]
    #[case("nil;", vec![()], "")]
    #[case("print nil;", vec![()], "nil\n")]
    #[case("print nil;\ntrue;", vec![(), ()], "nil\n")]
    #[case("true;", vec![()], "")]
    #[case("print 3 + 4; 10;", vec![(), ()], "7\n")]
    #[case("print 3 + 4; print \"hello\";", vec![(), ()], "7\nhello\n")]
    #[case("var x = 17; print x; var x = nil; print x;", vec![(), (), (), ()],"17\nnil\n")]
    #[case("var x = 17; var y = 13; x = y = 4; print x * y;", vec![(), (), (), ()],"16\n")]
    #[case("var x = 17; print x; x = nil; print x;", vec![(), (), (), ()],"17\nnil\n")]
    fn test_eval(
        #[case] input: &str,
        #[case] want: Vec<()>,
        #[case] want_stdout: &'static str,
    ) -> LoxResult<()> {
        let mut buf = vec![];
        let got = str_eval(input, &mut buf)?;
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
    #[case("x = 4;", "[line 1] Error: value error: undefined variable: 'x'", "")]
    fn test_eval_error(
        #[case] input: &str,
        #[case] want: &str,
        #[case] want_stdout: &str,
    ) -> LoxResult<()> {
        let mut buf = vec![];
        let got = str_eval(input, &mut buf).expect_err("should have created an error");

        assert_eq!(format!("{got}"), want);

        assert_eq!(std::str::from_utf8(&buf), Ok(want_stdout));
        Ok(())
    }
}
