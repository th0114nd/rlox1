use crate::expr::Expr;
use crate::token::TokenType::*;
use crate::value::Value;
use crate::value::ValueError;

impl<'a> Expr<'a> {
    fn eval(&self) -> Result<Value, ValueError> {
        match self {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Grouping(expr) => expr.eval(),
            Expr::Unary { operator, right } => {
                let right = right.eval()?;
                match operator.token {
                    Minus => -right,
                    Bang => Ok(Value::Bool(!bool::from(right))),
                    _ => panic!("invalid unary operator '{}'", operator.lexeme),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = left.eval()?;
                let right = right.eval()?;
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
    use Value::*;

    #[rstest::rstest]
    #[case("nil", VNil)]
    #[case("true", Bool(true))]
    #[case("false", Bool(false))]
    #[case("3.14", VNumber(3.14))]
    #[case("  \" a string \" ", VString(" a string ".to_owned()))]
    fn test_eval(#[case] input: &str, #[case] want: Value) -> LoxResult<()> {
        let mut scanner = crate::scanner::Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = crate::parser::Parser::new(&tokens);
        let expr = parser.parse()?;
        let got = expr.eval()?;
        assert_eq!(got, want);
        Ok(())
    }
}
