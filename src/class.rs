use crate::callable::LoxCallable;
use crate::error::RuntimeError;
use crate::interpreter::Interpreter;
use crate::value::Value;
use compact_str::CompactString;
use std::fmt;
use std::rc::Rc;
//pub trait AnyClass: fmt::Display + fmt::Debug {}
// Is every object callable? or just classes?
// I guess we could always panic?
pub trait AnyClass: fmt::Display + fmt::Debug + LoxCallable {}
//struct LoxClass {
//
//},h
//j
#[derive(Debug, Clone)]
pub struct LoxClass {
    pub name: CompactString,
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl LoxCallable for LoxClass {
    fn arity(&self) -> usize {
        // TODO: what is this actually?
        0
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let value = LoxInstance {
            class: self.clone(),
        };
        Ok(Value::Object(Rc::new(value)))
    }
}

impl AnyClass for LoxClass {}

#[derive(Debug)]
pub struct LoxInstance {
    // Clone for now, but this should maybe be refcounted
    class: LoxClass,
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} instance", self.class)
    }
}

impl LoxCallable for LoxInstance {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        panic!("cannot call a class instance")
    }
}

impl AnyClass for LoxInstance {}
