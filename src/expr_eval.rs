use crate::environment::Env;
use crate::error::RuntimeError;
use crate::interpreter::Interpreter;
use crate::models::Expr;
use crate::models::TokenType::*;
use crate::models::Value;

impl Interpreter {
    pub fn eval_expr(&mut self, line: usize, expr: &Expr) -> Result<Value, RuntimeError> {
        self.priv_eval(line, expr)
    }

    // PUT THE LINE NUMBER ON EXPR
    fn priv_eval(&mut self, line: usize, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Variable(token) => {
                let name = &token.lexeme;
                let expr_ptr = expr as *const Expr;
                let depth = self.resolutions.get(&expr_ptr);
                match depth {
                    None => self.globals.get(name),
                    Some(depth) => self.environment.get_at(name, *depth),
                }
            }
            Expr::Assign { name, value } => {
                let name = &name.lexeme;
                let right = self.priv_eval(line, value)?;
                let expr_ptr = expr as *const Expr;
                let depth = self.resolutions.get(&expr_ptr);
                match depth {
                    None => self.globals.assign(name, right.clone())?,
                    Some(depth) => self.environment.assign_at(name, right.clone(), *depth)?,
                }
                Ok(right)
            }
            Expr::Grouping(expr) => self.priv_eval(line, expr),
            Expr::Unary { operator, right } => {
                let right = self.priv_eval(line, right)?;
                match operator.token {
                    Minus => -right,
                    Bang => Ok(Value::Bool(!bool::from(right))),
                    // ok to panic -- we should never parse a different unary op
                    _ => panic!("invalid unary operator '{}'", operator.lexeme),
                }
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.priv_eval(line, left)?;
                match (bool::from(&left), &operator.token) {
                    (false, &And) => Ok(left),
                    (true, &Or) => Ok(left),
                    (true, &And) | (false, Or) => self.priv_eval(line, right),
                    // ok to panic -- we should never parse a different logical op
                    _ => panic!("invalid logical operator '{}'", operator.lexeme),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.priv_eval(line, left)?;
                let right = self.priv_eval(line, right)?;
                match operator.token {
                    Plus => left + right,
                    Minus => left - right,
                    Star => left * right,
                    Slash => left / right,
                    BangEqual => Ok(Value::Bool(left != right)),
                    EqualEqual => Ok(Value::Bool(left == right)),
                    Less => Ok(Value::Bool(left < right)),
                    LessEqual => Ok(Value::Bool(left <= right)),
                    Greater => Ok(Value::Bool(left > right)),
                    GreaterEqual => Ok(Value::Bool(left >= right)),
                    // ok to panic -- we should never parse a different binary op
                    _ => panic!("invalid operation {operator}"),
                }
            }
            Expr::Call { callee, arguments } => {
                let callee: Value = self.priv_eval(line, callee)?;
                let arguments: Vec<Value> = arguments
                    .iter()
                    .map(|arg| self.priv_eval(line, arg))
                    .collect::<Result<Vec<Value>, RuntimeError>>()?;
                if let Value::Callable(callee) = callee {
                    let arity = callee.arity();
                    if arity != arguments.len() {
                        return Err(RuntimeError::ArityMismatch {
                            line: "TODO".into(),
                            want: arity,
                            got: arguments.len(),
                        })?;
                    }
                    Ok(callee.call(self, arguments)?)
                } else {
                    Err(RuntimeError::NonCallableCalled {
                        line: "TODO".into(),
                        value: callee,
                    })?
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Env;
    use crate::error::LoxError;
    use crate::error::LoxResult;
    use crate::parser::Parser;
    use crate::scanner::Scanner;
    use Value::*;

    fn str_eval(input: &str, interpreter: &mut Interpreter) -> LoxResult<Value> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let expr = parser.expression().map_err(|e| vec![e])?;
        interpreter.eval_expr(107, &expr).map_err(LoxError::from)
    }

    #[rstest::rstest]
    #[case("nil", VNil)]
    #[case("true", Bool(true))]
    #[case("false", Bool(false))]
    #[case("3.14", VNumber(3.14))]
    #[case("  \" a string \" ", VString(" a string ".into()))]
    #[case("2 + 4 + 5 * 3", VNumber(21.0))]
    #[case("\"foo\" + \"bar\"", VString("foobar".into()))]
    #[case("4 - 7.5", VNumber(-3.5))]
    #[case("4 == 5", Bool(false))]
    #[case("!4", Bool(false))]
    #[case("!nil", Bool(true))]
    #[case("0 - -7", VNumber(7.0))]
    #[case(r#""lox" == "lo" + "x""#, Bool(true))]
    #[case("false or false", Bool(false))]
    #[case("true or false", Bool(true))]
    #[case("false or true", Bool(true))]
    #[case("true or true", Bool(true))]
    #[case("false and false", Bool(false))]
    #[case("true and false", Bool(false))]
    #[case("false and true", Bool(false))]
    #[case("true and true", Bool(true))]
    #[case("nil and hello", VNil)]
    #[case("nil or 17", VNumber(17.0))]
    fn test_eval(#[case] input: &str, #[case] want: Value) -> LoxResult<()> {
        //let mut env = Environment::default();
        let mut interpreter = Interpreter::default();
        let got = str_eval(input, &mut interpreter)?;
        assert_eq!(got, want);
        Ok(())
    }

    #[rstest::rstest]
    #[case("4 + \"lox\"", "[line TODO] Error: type mismatch: 4 vs lox")]
    #[case("2 + something", "[line TODO] Error: undefined variable: 'something'")]
    fn test_eval_error(#[case] input: &str, #[case] want: &str) -> LoxResult<()> {
        let mut interpreter = Interpreter::default();
        let got = str_eval(input, &mut interpreter).expect_err("should not evaluated");
        assert_eq!(format!("{got}"), want);
        Ok(())
    }

    #[rstest::rstest]
    #[case("defined + 1", VNumber(82.0))]
    fn test_eval_env(#[case] input: &str, #[case] want: Value) -> LoxResult<()> {
        let mut interpreter = Interpreter::default();
        interpreter.environment.define("defined", VNumber(81.0));
        let got = str_eval(input, &mut interpreter)?;
        assert_eq!(got, want);
        Ok(())
    }
}
