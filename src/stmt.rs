use crate::expr::Expr;
use std::fmt;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct StmtList<'a>(pub Vec<Stmt<'a>>);

impl<'a> fmt::Display for StmtList<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for stmt in self.0.iter() {
            writeln!(f, "{stmt}")?;
        }
        Ok(())
    }
}
