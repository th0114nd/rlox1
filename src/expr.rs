use crate::token::Token;
use crate::value;
use std::fmt;

// What can we do with an expr?
#[derive(Debug)]
pub enum Expr {
    Literal(value::Value),
    Variable(Token),
    This(Token),
    Super(Token, Token),
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Expr::Literal(value) => write!(f, "{}", value),
            Expr::This(_) => write!(f, "this"),
            Expr::Super(_, method) => write!(f, "super.{}", method.lexeme),
            Expr::Variable(token) => write!(f, "v#{}", token.lexeme),
            Expr::Assign { name, value } => write!(f, "(= v#{} {value})", name.lexeme),
            Expr::Grouping(gr) => write!(f, "(group {})", gr),
            Expr::Unary { operator, right } => write!(f, "({} {right})", operator.lexeme),
            Expr::Binary {
                left,
                operator,
                right,
            } => write!(f, "({} {left} {right})", operator.lexeme),
            Expr::Logical {
                left,
                operator,
                right,
            } => write!(f, "({} {left} {right})", operator.lexeme),
            Expr::Call { callee, arguments } => {
                write!(f, "({callee}")?;
                for arg in arguments {
                    write!(f, " {arg}")?;
                }
                write!(f, ")")
            }
            Expr::Get { object, name } => {
                write!(f, "(get {object} {})", name.lexeme)
            }
            Expr::Set {
                object,
                name,
                value,
            } => {
                write!(f, "(set {object} {} {value})", name.lexeme)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Expr::*;
    use super::*;
    use crate::token::TokenType;

    #[rstest::rstest]
    //#[case(Literal(Token{token: TokenType::Star, lexeme:"*", line: 1}), "*")]
    #[case(Literal(value::Value::VNil), "nil")]
    #[case(Binary{
        left: Box::new(
            Unary{
                operator: Token{token: TokenType::Minus, lexeme: "-".into(), line: 1},
                right: Box::new(Literal(value::Value::VNumber(123.0))),
            },
        ),
        operator: Token{token: TokenType::Star, lexeme: "*".into(), line: 1},
        right: Box::new(Grouping(Box::new(Literal(value::Value::VNumber(45.67))))),
    }, "(* (- 123) (group 45.67))")]
    fn test_display(#[case] expr: Expr, #[case] want: &str) {
        let got = format!("{}", expr);
        assert_eq!(got, want);
    }
}
