use crate::models::Expr;
use crate::models::Token;
use std::fmt;

#[derive(Debug)]
pub enum Stmt<'a> {
    Expr(usize, Expr<'a>),
    Print(usize, Expr<'a>),
    VarDecl(usize, Token<'a>, Option<Expr<'a>>),
    FunDecl {
        line: usize,
        name: Token<'a>,
        parameters: Vec<Token<'a>>,
        body: Box<Stmt<'a>>,
    },
    Block(Vec<Stmt<'a>>),
    IfThenElse {
        line: usize,
        if_expr: Expr<'a>,
        then_stmt: Box<Stmt<'a>>,
        else_stmt: Option<Box<Stmt<'a>>>,
    },
    While(usize, Expr<'a>, Box<Stmt<'a>>),
}

impl<'a> fmt::Display for Stmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stmt::Expr(_, expr) => write!(f, "expr({expr})"),
            Stmt::Print(_, expr) => write!(f, "print({expr})"),
            Stmt::VarDecl(_, token, expr) => match expr {
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
                line: _,
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
            Stmt::While(_, expr, stmt) => write!(f, "(while {expr} {stmt})"),
            Stmt::FunDecl {
                line: _,
                name,
                parameters,
                body,
            } => {
                write!(f, "(defn {} '(", name.lexeme)?;
                for (i, parameter) in parameters.into_iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", parameter.lexeme)?;
                }
                write!(f, ") {body})")
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
