use crate::expr::Expr;
use std::fmt;

pub enum Stmt<'a> {
    Expr(Expr<'a>),
    Print(Expr<'a>),
}

impl<'a> fmt::Display for Stmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stmt::Expr(expr) => write!(f, "expr({})", expr),
            Stmt::Print(expr) => write!(f, "print({})", expr),
        }
    }
}
