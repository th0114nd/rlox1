use crate::environment::Env;
use crate::interpreter::Interpreter;
use crate::value::Value;
use std::fmt;
use std::time;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum CallError {
    #[error("arity mismatch")]
    ArityMismatch(usize, usize),
    #[error("system time error")]
    SystemTimeError,
    #[error("not callable: {0}")]
    NonCallableCalled(String),
}

pub trait LoxCallable: fmt::Display + fmt::Debug {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, CallError>;
}

#[derive(Debug)]
pub struct Clock;

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "clock")
    }
}

impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _interpreter: &mut Interpreter, _args: Vec<Value>) -> Result<Value, CallError> {
        let now = time::SystemTime::now();
        let elapsed = now
            .duration_since(time::UNIX_EPOCH)
            .or(Err(CallError::SystemTimeError))?;
        Ok(Value::VNumber(elapsed.as_secs_f64()))
    }
}

use crate::models::FunDecl;
#[derive(Debug)]
pub struct LoxFunction(pub FunDecl);

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fn {}>", self.0.name.lexeme)
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.0.parameters.len()
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, CallError> {
        interpreter.environment.push();
        for (param, arg) in self.0.parameters.iter().zip(args) {
            interpreter.environment.define(&param.lexeme, arg);
        }
        interpreter
            .eval(&self.0.body)
            .expect("oops! fix the error cycle");
        interpreter.environment.pop();
        // TODO: parse return values
        Ok(Value::VNil)
    }
}
