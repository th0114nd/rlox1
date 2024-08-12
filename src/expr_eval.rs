use crate::error::LoxError;
use crate::error::LoxResult;
use crate::expr::Expr;
use crate::token::TokenType::*;
use crate::value::TypeMismatch;
use crate::value::Value;

impl<'a> Expr<'a> {
    fn eval(&self) -> Result<Value, TypeMismatch> {
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
