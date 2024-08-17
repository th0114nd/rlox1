use crate::error::LoxError;
use crate::expr_eval::Eval;
use environment::Env;
use environment::Environment;
use models::Stmt;
use models::Value;
use std::io;

pub trait SEval {
    fn eval(&self, w: &mut impl io::Write, env: &mut Environment) -> Result<(), LoxError>;
}

impl<'a> SEval for &Stmt<'a> {
    fn eval(&self, w: &mut impl io::Write, env: &mut Environment) -> Result<(), LoxError> {
        match self {
            Stmt::Expr(line, expr) => {
                expr.eval(*line, env)?;
                Ok(())
            }
            Stmt::Print(line, expr) => {
                let v = expr.eval(*line, env)?;
                writeln!(w, "{v}").expect("writes should not fail");
                Ok(())
            }
            Stmt::VarDecl(line, token, expr) => {
                let value = match expr {
                    None => Value::VNil,
                    Some(expr) => expr.eval(*line, env)?,
                };
                env.define(token.lexeme, value);
                Ok(())
            }
            Stmt::Block(stmts) => env.fork(|env| {
                for s in stmts {
                    s.eval(w, env)?;
                }
                Ok(())
            }),
            Stmt::IfThenElse {
                line,
                if_expr,
                then_stmt,
                else_stmt,
            } => {
                let cond = if_expr.eval(*line, env)?;
                if bool::from(cond) {
                    // why not just keep line numbers on the statements ?
                    then_stmt.as_ref().eval(w, env)
                } else {
                    match else_stmt {
                        None => Ok(()),
                        // You're just guessing line numbers now
                        Some(else_stmt) => else_stmt.as_ref().eval(w, env),
                    }
                }
            }
            Stmt::While(line, expr, stmt) => {
                while bool::from(expr.eval(*line, env)?) {
                    stmt.as_ref().eval(w, env)?;
                }
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

    fn str_eval(input: &str, buf: &mut Vec<u8>) -> LoxResult<()> {
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
    #[case("nil;", "")]
    #[case("print nil;", "nil\n")]
    #[case("print nil;\ntrue;", "nil\n")]
    #[case("true;", "")]
    #[case("print 3 + 4; 10;", "7\n")]
    #[case("print 3 + 4; print \"hello\";", "7\nhello\n")]
    #[case("var x = 17; print x; var x = nil; print x;", "17\nnil\n")]
    #[case("var x = 17; var y = 13; x = y = 4; print x * y;", "16\n")]
    #[case("var x = 17; print x; x = nil; print x;", "17\nnil\n")]
    #[case("if (\"hello\") print 4;", "4\n")]
    #[case("if (nil) print 4;", "")]
    #[case("if (nil) print 4; else print 3;", "3\n")]
    #[case("var i = 0; while (i < 4) {i = i + 1; print i;}", "1\n2\n3\n4\n")]
    fn test_eval(#[case] input: &str, #[case] want_stdout: &'static str) -> LoxResult<()> {
        let mut buf = vec![];
        let _ = str_eval(input, &mut buf)?;

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

    #[test]
    fn test_local_script() -> LoxResult<()> {
        let input = r#"
var a = "global a";
var b = "global b";
var c = "global c";
{
  var a = "outer a";
  var b = "outer b";
  {
    var a = "inner a";
    print a;
    print b;
    print c;
  }
  print a;
  print b;
  print c;
}
print a;
print b;
print c;
"#;

        let want = r#"inner a
outer b
global c
outer a
outer b
global c
global a
global b
global c
"#;
        let mut buf = vec![];
        let _ = str_eval(input, &mut buf)?;
        assert_eq!(std::str::from_utf8(&buf).unwrap(), want);
        Ok(())
    }
}
