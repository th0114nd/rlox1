use crate::token::Token;
use crate::value;
use std::fmt;

// What can we do with an expr?
pub enum Expr<'a> {
    Literal(value::Value),
    Grouping(Box<Expr<'a>>),
    // TODO: this should be larger than just tokens, maybe it should include class literals as
    // well?
    // can we do less broad than Box<dyn Any>?
    Unary {
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    Binary {
        left: Box<Expr<'a>>,
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
}

impl<'a> fmt::Display for Expr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Expr::Literal(value) => write!(f, "{}", value),
            Expr::Grouping(gr) => write!(f, "(group {})", gr),
            Expr::Unary { operator, right } => write!(f, "({} {right})", operator.lexeme),
            Expr::Binary {
                left,
                operator,
                right,
            } => write!(f, "({} {left} {right})", operator.lexeme),
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
    #[case(Literal(value::Value::Nil), "nil")]
    #[case(Binary{
        left: Box::new(
            Unary{
                operator: Token{token: TokenType::Minus, lexeme: "-", line: 1},
                right: Box::new(Literal(value::Value::Number(123.0))),
            },
        ),
        operator: Token{token: TokenType::Star, lexeme: "*", line: 1},
        right: Box::new(Grouping(Box::new(Literal(value::Value::Number(45.67))))),
    }, "(* (- 123) (group 45.67))")]
    fn test_display(#[case] expr: Expr, #[case] want: &str) {
        let got = format!("{}", expr);
        assert_eq!(got, want);
    }
}
