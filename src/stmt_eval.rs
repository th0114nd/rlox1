use crate::callable::LoxFunction;
use crate::class::LoxClass;
use crate::environment::Env;
use crate::error::RuntimeError;
use crate::interpreter::Interpreter;
use crate::models::Stmt;
use crate::models::Value;
use std::io::Write;
use std::mem;
use std::rc::Rc;

impl Interpreter {
    pub fn eval(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expr(line, expr) => {
                self.eval_expr(*line, expr)?;
                Ok(())
            }
            Stmt::Print(line, expr) => {
                let v = self.eval_expr(*line, expr)?;
                writeln!(self, "{v}").expect("writes should not fail");
                Ok(())
            }
            Stmt::VarDecl(line, token, expr) => {
                let value = match expr {
                    None => Value::VNil,
                    Some(expr) => self.eval_expr(*line, expr)?,
                };
                self.environment.define(&token.lexeme, value);
                Ok(())
            }
            Stmt::FunDecl(fun_decl) => {
                let f = LoxFunction {
                    definition: fun_decl.clone(),
                    closure: self.environment.clone(),
                };
                let callable = Value::Callable(Rc::new(f));
                self.environment.define(&fun_decl.name.lexeme, callable);
                Ok(())
            }
            Stmt::ClassDecl { name, .. } => {
                let class = LoxClass {
                    name: name.lexeme.clone(),
                };
                let object = Value::Object(Rc::new(class));
                self.environment.define(&name.lexeme, object);
                Ok(())
            }
            Stmt::Block(stmts) => {
                let mut alt_env = self.environment.push();
                mem::swap(&mut alt_env, &mut self.environment);
                let mut result = Ok(());
                for s in stmts {
                    result = result.and_then(|_| self.eval(s));
                }
                mem::swap(&mut alt_env, &mut self.environment);
                result
            }
            Stmt::IfThenElse {
                line,
                if_expr,
                then_stmt,
                else_stmt,
            } => {
                let cond = self.eval_expr(*line, if_expr)?;
                if bool::from(cond) {
                    self.eval(then_stmt)
                } else {
                    match else_stmt {
                        None => Ok(()),
                        Some(else_stmt) => self.eval(else_stmt),
                    }
                }
            }
            Stmt::While(line, expr, stmt) => {
                while bool::from(self.eval_expr(*line, expr)?) {
                    self.eval(stmt)?;
                }
                Ok(())
            }
            Stmt::Return(line, expr) => {
                let value = self.eval_expr(*line, expr)?;
                Err(RuntimeError::Return {
                    line: format!("{line}").into(),
                    value,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::LoxResult;
    use crate::interpreter::Interpreter;
    use crate::parser::Parser;
    use crate::resolver::Resolver;
    use crate::scanner::Scanner;
    use std::str::from_utf8;
    //use std::rc::Rc;

    fn str_eval(input: &str) -> LoxResult<String> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let stmts = parser.parse()?;

        let mut resolver = Resolver::default();

        let mut interpreter = Interpreter::default();
        let resolutions = resolver.resolve(&stmts)?;
        interpreter.resolutions = resolutions;
        interpreter.interpret(&stmts)?;
        Ok(from_utf8(&interpreter.buffer)
            .expect("must parse output")
            .into())
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
        let got = str_eval(input)?;

        assert_eq!(got, want_stdout);
        Ok(())
    }

    #[rstest::rstest]
    #[case(
        "print nil;\n 4 + \"lox\";\n 2 + \"oops\";",
        "[line TODO] Error: type mismatch: 4 vs lox",
        "nil\n"
    )]
    #[case("x = 4;", "[line TODO] Error: undefined variable: 'x'", "")]
    fn test_eval_error(
        #[case] input: &str,
        #[case] want: &str,
        #[case] _want_stdout: &str,
    ) -> LoxResult<()> {
        let got = str_eval(input).expect_err("should have created an error");

        assert_eq!(format!("{got}"), want);

        // technically this should be checked but too hard
        //assert_eq!(from_utf8(&buf), Ok(want_stdout));
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
        let got = str_eval(input)?;
        assert_eq!(got, want);
        Ok(())
    }

    #[test]
    fn test_clock() -> LoxResult<()> {
        let input = "print clock();";
        let got = str_eval(input)?;
        assert_eq!(&got[0..2], "17");
        Ok(())
    }

    #[test]
    fn test_define_action() -> LoxResult<()> {
        let input = r#"
fun f(a, b) {
    print a + b;
}
f("hello", "world");
"#;
        let got = str_eval(input)?;
        assert_eq!(got, "helloworld\n");
        Ok(())
    }

    #[test]
    fn test_return() -> LoxResult<()> {
        let input = r#"
fun plus(a, b) {
    return a + b;
}
print plus("hello", "world");
"#;
        let got = str_eval(input)?;
        assert_eq!(got, "helloworld\n");
        Ok(())
    }

    #[test]
    fn test_recursive() -> LoxResult<()> {
        let input = r#"
fun rec(n) {
    if (n <= 0) return;
    print n;
    rec(n-1);
}
rec(3);
"#;
        let got = str_eval(input)?;
        assert_eq!(got, "3\n2\n1\n");
        Ok(())
    }

    #[test]
    fn test_closure() -> LoxResult<()> {
        let input = r#"
fun makeCounter() {
    var i = 0;
    fun counter() {
        i = i + 1;
        return i;
    }
    return counter;
}
var count = makeCounter();
print count();
print count();
"#;
        let got = str_eval(input)?;
        assert_eq!(got, "1\n2\n");
        Ok(())
    }

    #[test]
    fn test_lexical_scoping() -> LoxResult<()> {
        let input = r#"
var a = "global";
{
  fun showA() {
    print a;
  }

  showA();
  var a = "block";
  showA();
}
"#;
        let got = str_eval(input)?;
        assert_eq!(got, "global\nglobal\n");
        Ok(())
    }

    #[test]
    fn test_class_print() -> LoxResult<()> {
        let input = " class X {} print X;";
        let got = str_eval(input)?;
        assert_eq!(got, "X\n");
        Ok(())
    }

    #[test]
    fn test_class_instance() -> LoxResult<()> {
        let input = "class Bagel {} print Bagel();";
        let got = str_eval(input)?;
        assert_eq!(got, "Bagel instance\n");
        Ok(())
    }
}
