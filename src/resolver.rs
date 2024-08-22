use crate::models::Expr;
use crate::models::FunDecl;
use crate::models::Stmt;
use crate::models::StmtList;
use crate::models::Value;
use std::collections::HashMap;

use crate::models::Token;
use compact_str::CompactString;
use std::mem;
use thiserror::Error;

#[derive(PartialEq, Debug, Error)]
pub enum ResolverError {
    #[error("variable accessed before definition: {0}")]
    AccessBeforeInit(Token),
    #[error("variable redefined: {0}")]
    AlreadyDefined(Token),

    #[error("return outside of function: {0}")]
    NoFuncReturn(String),

    #[error("this outside of class: {0}")]
    NoClassThis(Token),
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
enum FuncType {
    #[default]
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
enum ClassType {
    #[default]
    None,
    Class,
}

#[derive(Debug, Default)]
pub struct Resolver {
    resolutions: HashMap<*const Expr, usize>,
    errors: Vec<ResolverError>,
    scopes: Vec<HashMap<CompactString, bool>>,
    func_type: FuncType,
    class_type: ClassType,
}

impl Resolver {
    pub fn resolve(
        &mut self,
        stmt_list: &StmtList,
    ) -> Result<HashMap<*const Expr, usize>, Vec<ResolverError>> {
        self.resolve_stmts(stmt_list);
        if self.errors.is_empty() {
            Ok(mem::take(&mut self.resolutions))
        } else {
            Err(mem::take(&mut self.errors))
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(Default::default());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, token: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            let name = token.lexeme.clone();
            match scope.get(&name) {
                Some(_) => self
                    .errors
                    .push(ResolverError::AlreadyDefined(token.clone())),
                None => {
                    scope.insert(name, false);
                }
            }
            scope.insert(token.lexeme.clone(), false);
        }
    }

    fn define(&mut self, token: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(token.lexeme.clone(), true);
        }
    }

    fn resolve_local(&mut self, expr: &Expr, token: &Token) {
        for (offset, scope) in self.scopes.iter().rev().enumerate() {
            match scope.get(&token.lexeme) {
                None => continue,
                Some(false) => {
                    self.errors
                        .push(ResolverError::AccessBeforeInit(token.clone()));
                    return;
                }
                Some(true) => {
                    let expr_ptr = expr as *const Expr;
                    self.resolutions.insert(expr_ptr, offset);
                    return;
                }
            }
        }
    }

    fn resolve_stmts(&mut self, stmts: &StmtList) {
        for stmt in stmts.0.iter() {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_function(&mut self, func_type: FuncType, fun_decl: &FunDecl) {
        let FunDecl {
            name,
            parameters,
            body,
            ..
        } = fun_decl;
        let enclosing_function = self.func_type;
        self.func_type = func_type;
        self.declare(name);
        self.define(name);
        self.begin_scope();
        for parameter in parameters {
            self.declare(parameter);
            self.define(parameter);
        }
        self.resolve_stmts(body);
        self.end_scope();
        self.func_type = enclosing_function;
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(_, expr) => self.resolve_expr(expr),
            Stmt::Print(_, expr) => self.resolve_expr(expr),
            Stmt::VarDecl(_, token, expr) => {
                self.declare(token);
                if let Some(expr) = expr {
                    self.resolve_expr(expr);
                }
                self.define(token);
            }
            Stmt::FunDecl(fun_decl) => self.resolve_function(FuncType::Function, fun_decl),
            Stmt::ClassDecl { name, methods, .. } => {
                let enclosing_class = self.class_type;
                self.class_type = ClassType::Class;
                self.declare(name);
                self.define(name);
                self.begin_scope();
                self.scopes.last_mut().unwrap().insert("this".into(), true);

                for method in methods {
                    let func_type = if method.name.lexeme == "init" {
                        FuncType::Initializer
                    } else {
                        FuncType::Method
                    };
                    self.resolve_function(func_type, method);
                }
                self.end_scope();
                self.class_type = enclosing_class;
            }
            Stmt::Block(stmts) => {
                self.begin_scope();
                self.resolve_stmts(stmts);
                self.end_scope();
            }
            Stmt::IfThenElse {
                line: _,
                if_expr,
                then_stmt,
                else_stmt,
            } => {
                self.resolve_expr(if_expr);
                self.resolve_stmt(then_stmt);
                if let Some(else_stmt) = else_stmt {
                    self.resolve_stmt(else_stmt);
                }
            }
            Stmt::While(_, expr, stmt) => {
                self.resolve_expr(expr);
                self.resolve_stmt(stmt);
            }
            Stmt::Return(_, expr) => {
                if !matches!(expr, Expr::Literal(Value::VNil))
                    && matches!(self.func_type, FuncType::None | FuncType::Initializer)
                {
                    self.errors
                        .push(ResolverError::NoFuncReturn(format!("{expr}")));
                }
                self.resolve_expr(expr);
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        use Expr::*;
        match expr {
            Literal(_) => {}
            Variable(token) => self.resolve_local(expr, token),
            This(token) => {
                if self.class_type == ClassType::None {
                    self.errors.push(ResolverError::NoClassThis(token.clone()))
                }
                self.resolve_local(expr, token);
            }
            Assign { name, value } => {
                self.resolve_expr(value);
                self.resolve_local(expr, name)
            }
            Grouping(expr) => self.resolve_expr(expr),
            Unary { right, .. } => self.resolve_expr(right),
            Binary { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Logical { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Call { callee, arguments } => {
                self.resolve_expr(callee);
                for argument in arguments {
                    self.resolve_expr(argument);
                }
            }
            Get { object, .. } => {
                self.resolve_expr(object);
                // we can't statically resolve fields because the language is dynamic
            }
            Set { object, value, .. } => {
                self.resolve_expr(object);
                self.resolve_expr(value);
                // we can't statically resolve fields because the language is dynamic
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::LoxError;
    use crate::parser::Parser;
    use crate::scanner::Scanner;

    const SCRIPT_1: &str = r#"
var a = "global";
{
  fun showA() {
    print a;
  }

  showA();
  var a = "block";
  showA();
}
"#;

    #[rstest::rstest]
    // global variables are ignored
    #[case("var a = 17; print a + 1;", vec![])]
    // defined in same block
    #[case("{ var a = 19; print a + 2; }", vec![0])]
    // defined in parent global
    #[case("var a = 20; { print a + 2; }", vec![])]
    // defined in parent block
    #[case("{ var a = 20; { print a + 2; }}", vec![1])]
    // function parameter
    #[case("fun f(a) { print a; }", vec![0])]
    // function parameter with global
    #[case("var a; fun f(a) { print a; }", vec![0])]
    #[case("var a; var b; fun f(a, c) { print a; print b; print c; }", vec![0, 0])]
    #[case("{var a; var b; fun f(a, c) { print a; print b; print c; }}", vec![0, 0, 1])]
    #[case("{ var a; print a; { print a; var a; print a;}}", vec![0, 0, 1])]
    // two references to showA in block 2
    #[case(SCRIPT_1, vec![0, 0])]
    fn test_resolution(
        #[case] input: &str,
        #[case] want_depths: Vec<usize>,
    ) -> Result<(), LoxError> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let stmts = parser.parse()?;
        let mut resolver = Resolver::default();
        let resolutions = resolver.resolve(&stmts)?;
        let mut got_depths: Vec<_> = resolutions.values().cloned().collect();
        got_depths.sort();
        assert_eq!(got_depths, want_depths);
        Ok(())
    }

    // TODO: test resolver error cases
}
