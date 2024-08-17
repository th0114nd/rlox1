use crate::expr::Expr;
use crate::token::Token;
use std::fmt;

#[derive(Debug)]
pub enum Stmt<'a> {
    Expr(Expr<'a>),
    Print(Expr<'a>),
    VarDecl(Token<'a>, Option<Expr<'a>>),
    Block(Vec<Stmt<'a>>),
    IfThenElse {
        if_expr: Expr<'a>,
        then_stmt: Box<Stmt<'a>>,
        else_stmt: Option<Box<Stmt<'a>>>,
    },
}

impl<'a> fmt::Display for Stmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stmt::Expr(expr) => write!(f, "expr({expr})"),
            Stmt::Print(expr) => write!(f, "print({expr})"),
            Stmt::VarDecl(token, expr) => match expr {
                None => write!(f, "var({})", token.lexeme),
                Some(expr) => write!(f, "var({} = {expr})", token.lexeme),
            },
            Stmt::Block(stmts) => {
                writeln!(f, "{{")?;
                for stmt in stmts {
                    writeln!(f, "{stmt}")?;
                }
                write!(f, "}}")
            }
            Stmt::IfThenElse {
                if_expr,
                then_stmt,
                else_stmt,
            } => {
                write!(
                    f,
                    "(if {if_expr} {then_stmt} {})",
                    match else_stmt {
                        None => "{}".to_owned(),
                        Some(stmt) => format!("{stmt}"),
                    }
                )
            }
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
