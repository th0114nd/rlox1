//use crate::environment::Env;
//use crate::environment::Environment;
use crate::error::LoxError;
use environment::Env;
use environment::Environment;
//use crate::expr::Expr;
//use crate::token::TokenType::*;
//use crate::value::Value;
//use crate::value::ValueError;
use models::Expr;
use models::TokenType::*;
use models::Value;
use models::ValueError;
//use models::expr::Expr;
//

pub trait Eval {
    fn eval(&self, current: usize, env: &mut Environment) -> Result<Value, LoxError>;
    fn priv_eval(&self, env: &mut Environment) -> Result<Value, ValueError>;
}

impl<'a> Eval for Expr<'a> {
    fn eval(&self, current: usize, env: &mut Environment) -> Result<Value, LoxError> {
        match self.priv_eval(env) {
            Ok(v) => Ok(v),
            Err(value_error) => Err(LoxError::ValueError(current, value_error)),
        }
    }

    //impl<'a>

    fn priv_eval(&self, env: &mut Environment) -> Result<Value, ValueError> {
        match self {
            // TODO: no clone / refcount?
            Expr::Literal(value) => Ok(value.clone()),
            // TODO: no clone / refcout?
            Expr::Variable(token) => env.get(token.lexeme).cloned(),
            Expr::Assign { name, value } => {
                let right = value.priv_eval(env)?;
                // TODO: No clone / refcount?
                env.assign(name.lexeme, right.clone())?;
                Ok(right)
            }
            Expr::Grouping(expr) => expr.priv_eval(env),
            Expr::Unary { operator, right } => {
                let right = right.priv_eval(env)?;
                match operator.token {
                    Minus => -right,
                    Bang => Ok(Value::Bool(!bool::from(right))),
                    // ok to panic -- should never happen if written correctly
                    _ => panic!("invalid unary operator '{}'", operator.lexeme),
                }
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = bool::from(left.priv_eval(env)?);
                match (left, operator.token) {
                    (false, And) => Ok(Value::Bool(false)),
                    (true, Or) => Ok(Value::Bool(true)),
                    (true, And) | (false, Or) => right.priv_eval(env),
                    _ => panic!("invalid logical operator '{}'", operator.lexeme),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = left.priv_eval(env)?;
                let right = right.priv_eval(env)?;
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
                    // ok to panic -- should never happen if written correctly
                    _ => panic!("invalid operation {operator}"),
                }
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
    use Value::*;

    fn str_eval(input: &str, env: &mut Environment) -> LoxResult<Value> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let expr = parser.expression()?;
        expr.eval(107, env)
    }

    #[rstest::rstest]
    #[case("nil", VNil)]
    #[case("true", Bool(true))]
    #[case("false", Bool(false))]
    #[case("3.14", VNumber(3.14))]
    #[case("  \" a string \" ", VString(" a string ".to_owned()))]
    #[case("2 + 4 + 5 * 3", VNumber(21.0))]
    #[case("\"foo\" + \"bar\"", VString("foobar".to_string()))]
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
    fn test_eval(#[case] input: &str, #[case] want: Value) -> LoxResult<()> {
        let mut env = Environment::default();
        let got = str_eval(input, &mut env)?;
        assert_eq!(got, want);
        Ok(())
    }

    #[rstest::rstest]
    #[case(
        "4 + \"lox\"",
        "[line 107] Error: value error: type mismatch: 4 vs lox"
    )]
    #[case(
        "2 + something",
        "[line 107] Error: value error: undefined variable: 'something'"
    )]
    fn test_eval_error(#[case] input: &str, #[case] want: &str) -> LoxResult<()> {
        let mut env = Environment::default();
        let got = str_eval(input, &mut env).expect_err("should not evaluated");
        assert_eq!(format!("{got}"), want);
        Ok(())
    }

    #[rstest::rstest]
    #[case("defined + 1", VNumber(82.0))]
    fn test_eval_env(#[case] input: &str, #[case] want: Value) -> LoxResult<()> {
        let mut env = Environment::default();
        env.define("defined", VNumber(81.0));
        let got = str_eval(input, &mut env)?;
        assert_eq!(got, want);
        Ok(())
    }
}
