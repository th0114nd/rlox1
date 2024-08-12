use std::convert::TryFrom;
use std::fmt;
use std::rc::Rc;

use crate::token::TokenType;

type OpOutput = Result<Value, TypeMismatch>;

pub trait AnyClass: fmt::Display + fmt::Debug {}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    VString(String),
    Bool(bool),
    Nil,
    Class(Rc<dyn AnyClass>),
}

use Value::*;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Number(x) => x.fmt(f),
            VString(s) => write!(f, "\"{s}\""),
            Bool(b) => b.fmt(f),
            Nil => write!(f, "nil"),
            Class(x) => x.fmt(f),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = TypeMismatch;

    fn try_from(value: Value) -> Result<f64, Self::Error> {
        if let Number(x) = value {
            Ok(x)
        } else {
            Err(TypeMismatch {})
        }
    }
}

impl<'a> From<TokenType<'a>> for Value {
    fn from(token_type: TokenType<'a>) -> Value {
        match token_type {
            TokenType::Number(n) => Number(n),
            TokenType::String(s) => VString(s.to_owned()),
            TokenType::True => Bool(true),
            TokenType::False => Bool(false),
            TokenType::Nil => Nil,
            _ => panic!("invalid value: {token_type:?}"),
        }
    }
}

pub struct TypeMismatch {}

impl std::ops::Add for Value {
    type Output = OpOutput;

    fn add(self, other: Value) -> Self::Output {
        match (self, other) {
            (Number(lhs), Number(rhs)) => Ok(Number(lhs + rhs)),
            (VString(lhs), VString(rhs)) => Ok(VString(lhs + &rhs)),
            _ => Err(TypeMismatch {}),
        }
    }
}

impl std::ops::Sub for Value {
    type Output = OpOutput;

    fn sub(self, other: Value) -> Self::Output {
        let lhs = f64::try_from(self)?;
        let rhs = f64::try_from(other)?;
        Ok(Number(lhs - rhs))
    }
}

impl std::ops::Div for Value {
    type Output = OpOutput;

    fn div(self, other: Value) -> Self::Output {
        let lhs = f64::try_from(self)?;
        let rhs = f64::try_from(other)?;
        //if rhs == 0 {
        //    Err(ZeroDivisionError{})
        //    }
        Ok(Number(lhs / rhs))
    }
}

impl std::ops::Neg for Value {
    type Output = OpOutput;

    fn neg(self) -> Self::Output {
        let rhs = f64::try_from(self)?;
        Ok(Number(-rhs))
    }
}

impl std::ops::Mul for Value {
    type Output = OpOutput;

    fn mul(self, other: Value) -> Self::Output {
        let lhs = f64::try_from(self)?;
        let rhs = f64::try_from(other)?;
        Ok(Number(lhs * rhs))
    }
}

impl From<Value> for bool {
    fn from(value: Value) -> bool {
        match value {
            Nil | Bool(false) => false,
            _ => true,
        }
    }
}

impl std::cmp::PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Number(lhs), Number(rhs)) => lhs.partial_cmp(rhs),
            (VString(lhs), VString(rhs)) => lhs.partial_cmp(rhs),
            (Bool(lhs), Bool(rhs)) => lhs.partial_cmp(rhs),
            (Nil, Nil) => Some(std::cmp::Ordering::Equal),
            _ => None,
        }
    }
}

impl std::cmp::PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Number(lhs), Number(rhs)) => lhs == rhs,
            (VString(lhs), VString(rhs)) => lhs == rhs,
            (Bool(lhs), Bool(rhs)) => lhs == rhs,
            (Nil, Nil) => true,
            _ => false,
        }
    }
}
