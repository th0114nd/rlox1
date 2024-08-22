use crate::class::LoxInstance;
use crate::environment::Env;
use crate::environment::Environment;
use crate::error::RuntimeError;
use crate::interpreter::Interpreter;
use crate::value::Value;
use std::fmt;
use std::mem;
use std::rc::Rc;
use std::time;

pub trait LoxCallable: fmt::Display + fmt::Debug {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError>;
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

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let now = time::SystemTime::now();
        let elapsed =
            now.duration_since(time::UNIX_EPOCH)
                .or(Err(RuntimeError::SystemTimeError {
                    line: "TODO".into(),
                }))?;
        Ok(Value::VNumber(elapsed.as_secs_f64()))
    }
}

use crate::models::FunDecl;
#[derive(Debug, Clone)]
pub struct LoxFunction {
    pub definition: Rc<FunDecl>,
    pub closure: Rc<Environment>,
    pub is_init: bool,
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fn {}>", self.definition.name.lexeme)
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.definition.parameters.len()
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let mut closure = self.closure.push();
        mem::swap(&mut closure, &mut interpreter.environment);
        for (param, arg) in self.definition.parameters.iter().zip(args) {
            interpreter.environment.define(&param.lexeme, arg);
        }

        let result = interpreter.interpret(&self.definition.body);
        mem::swap(&mut closure, &mut interpreter.environment);
        match result {
            Ok(()) => {
                if self.is_init {
                    closure.get_at("this", 0)
                } else {
                    Ok(Value::VNil)
                }
            }
            Err(RuntimeError::Return { value, .. }) => {
                if self.is_init {
                    closure.get_at("this", 0)
                } else {
                    Ok(value)
                }
            }
            Err(err) => Err(err),
        }
    }
}

impl LoxFunction {
    pub fn bind(&self, instance: Rc<LoxInstance>) -> LoxFunction {
        let closure = self.closure.push();

        closure.define("this", Value::Object(Rc::new(instance)));
        LoxFunction {
            definition: self.definition.clone(),
            is_init: self.is_init,
            closure,
        }
    }
}
