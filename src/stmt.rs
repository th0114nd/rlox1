use crate::models::Expr;
use crate::models::Token;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct FunDecl {
    pub line: usize,
    pub name: Token,
    pub parameters: Vec<Token>,
    pub body: Rc<Stmt>,
}

#[derive(Debug)]
pub enum Stmt {
    Expr(usize, Expr),
    Print(usize, Expr),
    VarDecl(usize, Token, Option<Expr>),
    FunDecl(FunDecl),
    Block(Vec<Stmt>),
    IfThenElse {
        line: usize,
        if_expr: Expr,
        then_stmt: Box<Stmt>,
        else_stmt: Option<Box<Stmt>>,
    },
    While(usize, Expr, Box<Stmt>),
}

impl fmt::Display for Stmt {
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
            Stmt::FunDecl(FunDecl {
                line: _,
                name,
                parameters,
                body,
            }) => {
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
pub struct StmtList(pub Vec<Stmt>);

impl fmt::Display for StmtList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for stmt in self.0.iter() {
            writeln!(f, "{stmt}")?;
        }
        Ok(())
    }
}
