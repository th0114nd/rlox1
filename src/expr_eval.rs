use crate::error::LoxError;
use crate::expr::Expr;
use crate::token::TokenType::*;
use crate::value::Value;
use crate::value::ValueError;

impl<'a> Expr<'a> {
    pub fn eval(&self, current: usize) -> Result<Value, LoxError> {
        match self.priv_eval() {
            Ok(v) => Ok(v),
            Err(value_error) => Err(LoxError::ValueError(current, value_error)),
        }
    }

    fn priv_eval(&self) -> Result<Value, ValueError> {
        match self {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Grouping(expr) => expr.priv_eval(),
            Expr::Unary { operator, right } => {
                let right = right.priv_eval()?;
                match operator.token {
                    Minus => -right,
                    Bang => Ok(Value::Bool(!bool::from(right))),
                    // ok to panic -- should never happen if written correctly
                    _ => panic!("invalid unary operator '{}'", operator.lexeme),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = left.priv_eval()?;
                let right = right.priv_eval()?;
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
    fn test_eval(#[case] input: &str, #[case] want: Value) -> LoxResult<()> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let expr = parser.expression()?;
        let got = expr.eval(103)?;
        assert_eq!(got, want);
        Ok(())
    }

    #[rstest::rstest]
    #[case("4 + \"lox\"", "[line 1] Error: value error: type mismatch: 4 vs lox")]
    fn test_eval_error(#[case] input: &str, #[case] want: &str) -> LoxResult<()> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let expr = parser.expression()?;

        let got = expr.eval(1).expect_err("should not evaluate");
        assert_eq!(format!("{got}"), want);
        Ok(())
    }
}
