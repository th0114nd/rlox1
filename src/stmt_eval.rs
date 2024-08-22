use crate::callable::LoxFunction;
use crate::class::LoxClass;
use crate::environment::Env;
use crate::error::RuntimeError;
use crate::interpreter::Interpreter;
use crate::models::Stmt;
use crate::models::Value;
use std::collections::HashMap;
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
                    definition: fun_decl.clone().into(),
                    closure: self.environment.clone(),
                    is_init: false,
                };
                let callable = Value::Callable(Rc::new(f));
                self.environment.define(&fun_decl.name.lexeme, callable);
                Ok(())
            }
            Stmt::ClassDecl {
                name,
                methods,
                parent,
                line,
            } => {
                let parent_class = match parent {
                    None => None,
                    Some(p) => {
                        let maybe_class = self.eval_expr(*line, p)?;
                        match maybe_class {
                            Value::Class(lc) => Some(lc),
                            _ => panic!("not a class"),
                        }
                    }
                };
                let environment = match parent_class {
                    None => self.environment.clone(),
                    Some(ref lc) => {
                        let environment = self.environment.push();
                        environment.define("super", Value::Class(lc.clone()));
                        environment
                    }
                };

                let mut method_table = HashMap::default();
                for method in methods {
                    let name = method.name.lexeme.to_owned();
                    let m = LoxFunction {
                        definition: method.clone().into(),
                        closure: environment.clone(),
                        is_init: name == "init",
                    };
                    method_table.insert(name, m);
                }
                let class = LoxClass {
                    name: name.lexeme.clone(),
                    methods: method_table,
                    parent: parent_class,
                };
                let object = Value::Class(class.into());
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

    #[test]
    fn test_field_access() -> LoxResult<()> {
        let input = "class Bagel {}\n var b = Bagel(); b.greeting = \"world\"; print b.greeting;";
        let got = str_eval(input)?;
        assert_eq!(got, "world\n");
        Ok(())
    }

    #[test]
    fn test_nested_field_access() -> LoxResult<()> {
        let input = r#"
class Small {}
class Big {}
var s = Small();
var b = Big();
s.field = 95;
b.s = s;
b.s.field = 317;
print s.field;
print b.s.field;
"#;

        let got = str_eval(input)?;
        assert_eq!(got, "317\n317\n");
        Ok(())
    }

    #[test]
    fn test_method_simple() -> LoxResult<()> {
        let input = r#"
class Bacon {
  eat() {
    print "Crunch crunch crunch!";
  }
}

Bacon().eat();
"#;
        let got = str_eval(input)?;
        assert_eq!(got, "Crunch crunch crunch!\n");
        Ok(())
    }

    #[test]
    fn test_this() -> LoxResult<()> {
        let input = r#"
class Egotist {
  speak() {
    print this;
  }
}

var method = Egotist().speak;
method();
"#;
        let got = str_eval(input)?;
        assert_eq!(got, "Egotist instance\n");
        Ok(())
    }

    #[test]
    fn test_method_calls() -> LoxResult<()> {
        let input = r#"
class Person {
  sayName() {
    print this.name;
  }
}

var jane = Person();
jane.name = "Jane";

jane.sayName();
var method = jane.sayName;
method(); //
"#;
        let got = str_eval(input)?;
        assert_eq!(got, "Jane\nJane\n");
        Ok(())
    }

    #[test]
    fn test_invalid_this() -> LoxResult<()> {
        let input = "print this;";
        let got = str_eval(input).expect_err("should fail");
        assert_eq!(format!("{got}"), "this outside of class: This \"this\"");
        Ok(())
    }

    #[test]
    fn test_init() -> LoxResult<()> {
        let input = r#"
class Foo {
  init() {
    print this;
  }
}

var foo = Foo();
print foo.init();
"#;
        let got = str_eval(input)?;
        assert_eq!(got, "Foo instance\nFoo instance\nFoo instance\n");
        Ok(())
    }

    #[test]
    fn test_init_return() -> LoxResult<()> {
        let input = r#"
class Foo {
  init() {
    return "something else";
  }
}
"#;
        let got = str_eval(input).expect_err("should fail");
        assert_eq!(
            format!("{got}"),
            "return outside of function: something else"
        );
        Ok(())
    }

    #[test]
    fn test_inherits() -> LoxResult<()> {
        let input = r#"
class Doughnut {
  cook() {
    print "Fry until golden brown.";
  }
}

class BostonCream < Doughnut {}

BostonCream().cook();
"#;
        let got = str_eval(input)?;
        assert_eq!(got, "Fry until golden brown.\n");
        Ok(())
    }

    #[test]
    fn test_super() -> LoxResult<()> {
        let input = r#"
class Doughnut {
  cook() {
    print "Fry until golden brown.";
  }
}

class BostonCream < Doughnut {
  cook() {
    super.cook();
    print "Pipe full of custard and coat with chocolate.";
  }
}

BostonCream().cook();
"#;
        let got = str_eval(input)?;
        assert_eq!(
            got,
            "Fry until golden brown.\nPipe full of custard and coat with chocolate.\n"
        );
        Ok(())
    }

    #[test]
    fn test_super_stack() -> LoxResult<()> {
        let input = r#"
class A {
  method() {
    print "A method";
  }
}

class B < A {
  method() {
    print "B method";
  }

  test() {
    super.method();
  }
}

class C < B {}

C().test();
"#;
        let got = str_eval(input)?;
        assert_eq!(got, "A method\n");
        Ok(())
    }
}
