use crate::class::AnyClass;
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
        let mut alt_environment = self.closure.push();
        mem::swap(&mut alt_environment, &mut interpreter.environment);
        for (param, arg) in self.definition.parameters.iter().zip(args) {
            interpreter.environment.define(&param.lexeme, arg);
        }

        let result = interpreter.interpret(&self.definition.body);
        interpreter.environment = mem::take(&mut alt_environment);
        match result {
            Ok(()) => Ok(Value::VNil),
            Err(RuntimeError::Return { value, .. }) => Ok(value),
            Err(err) => Err(err),
        }
    }
}

impl LoxFunction {
    pub fn bind(&self, instance: Rc<LoxInstance>) -> LoxFunction {
        let mut closure = self.closure.push();

        closure.define("this", Value::Object(Rc::new(instance)));
        LoxFunction {
            definition: self.definition.clone(),
            closure,
        }
    }
}
