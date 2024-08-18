use crate::callable::CallError;
use crate::callable::LoxCallable;
use std::convert::TryFrom;
use std::fmt;
use std::rc::Rc;
use thiserror::Error;

use crate::token::TokenType;

// TODO
// MAKE TOKENS?STATEMENTS?EXPRS LIVE FOREVER AND STOP FIGHTING THE CHECKER
//
#[derive(Debug, Error, PartialEq)]
pub enum ValueError {
    #[error("type mismatch: {0} vs {1}")]
    TypeMismatch(Value, Value),
    #[error("zero division error")]
    ZeroDivError,
    #[error("undefined variable: '{0}'")]
    UndefinedVariable(String),
    #[error("call error: '{0}'")]
    CallError(#[from] CallError),
}

type OpOutput = Result<Value, ValueError>;

pub trait AnyClass: fmt::Display + fmt::Debug {}

#[derive(Debug, Clone, Default)]
pub enum Value {
    #[default]
    VNil,
    Bool(bool),
    VNumber(f64),
    VString(String),
    Callable(Box<dyn LoxCallable>),
    //Object(Rc<dyn AnyClass>),
}

use Value::*;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VNil => write!(f, "nil"),
            VNumber(x) => x.fmt(f),
            Bool(b) => b.fmt(f),
            VString(s) => write!(f, "{s}"),
            //Object(x) => x.fmt(f),
            Callable(_) => todo!(),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<f64, Self::Error> {
        if let VNumber(x) = value {
            Ok(x)
        } else {
            Err(ValueError::TypeMismatch(value, VNumber(1197.9)))
        }
    }
}

impl<'a> From<TokenType<'a>> for Value {
    fn from(token_type: TokenType<'a>) -> Value {
        match token_type {
            TokenType::TNumber(n) => VNumber(n),
            TokenType::TString(s) => VString(s.to_owned()),
            TokenType::True => Bool(true),
            TokenType::False => Bool(false),
            TokenType::Nil => VNil,
            _ => panic!("invalid value: {token_type:?}"),
        }
    }
}

impl std::ops::Add for Value {
    type Output = OpOutput;

    fn add(self, other: Value) -> Self::Output {
        match (self, other) {
            (VNumber(lhs), VNumber(rhs)) => Ok(VNumber(lhs + rhs)),
            (VString(lhs), VString(rhs)) => Ok(VString(lhs + &rhs)),
            (x, y) => Err(ValueError::TypeMismatch(x, y)),
        }
    }
}

impl std::ops::Sub for Value {
    type Output = OpOutput;

    fn sub(self, other: Value) -> Self::Output {
        let lhs = f64::try_from(self)?;
        let rhs = f64::try_from(other)?;
        Ok(VNumber(lhs - rhs))
    }
}

impl std::ops::Div for Value {
    type Output = OpOutput;

    fn div(self, other: Value) -> Self::Output {
        let lhs = f64::try_from(self)?;
        let rhs = f64::try_from(other)?;
        if rhs == 0.0 {
            Err(ValueError::ZeroDivError {})
        } else {
            Ok(VNumber(lhs / rhs))
        }
    }
}

impl std::ops::Neg for Value {
    type Output = OpOutput;

    fn neg(self) -> Self::Output {
        let rhs = f64::try_from(self)?;
        Ok(VNumber(-rhs))
    }
}

impl std::ops::Mul for Value {
    type Output = OpOutput;

    fn mul(self, other: Value) -> Self::Output {
        let lhs = f64::try_from(self)?;
        let rhs = f64::try_from(other)?;
        Ok(VNumber(lhs * rhs))
    }
}

impl From<Value> for bool {
    fn from(value: Value) -> bool {
        !matches!(value, VNil | Bool(false))
    }
}

impl From<&Value> for bool {
    fn from(value: &Value) -> bool {
        !matches!(value, VNil | Bool(false))
    }
}

impl std::cmp::PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (VNumber(lhs), VNumber(rhs)) => lhs.partial_cmp(rhs),
            (VString(lhs), VString(rhs)) => lhs.partial_cmp(rhs),
            (Bool(lhs), Bool(rhs)) => lhs.partial_cmp(rhs),
            (VNil, VNil) => Some(std::cmp::Ordering::Equal),
            _ => None,
        }
    }
}

impl std::cmp::PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (VNumber(lhs), VNumber(rhs)) => lhs == rhs,
            (VString(lhs), VString(rhs)) => lhs == rhs,
            (Bool(lhs), Bool(rhs)) => lhs == rhs,
            (VNil, VNil) => true,
            _ => false,
        }
    }
}
