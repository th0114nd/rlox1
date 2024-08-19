use crate::environment::Env;
use crate::environment::Environment;
use crate::error::RuntimeError;
use crate::interpreter::Interpreter;
use crate::value::Value;
use std::fmt;
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
#[derive(Debug)]
pub struct LoxFunction {
    pub definition: FunDecl,
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
        // we could stack the closure environment on top of the interpreter environment
        //use std::mem;
        //let initial_len = interpreter.environment.stack.len();
        //let initial_environment = interpreter.environment;
        use std::mem;

        //mem::swap(&mut interpreter.environment, &mut self.closure);

        // & MUT IS NOT INTERIOR MUTABILITY
        //interpreter.environment.stack.append(&mut self.1.stack);
        //for env_map in self.1.stack.iter() {
        //    interpreter.environment.stack.push(env_map.clone());
        //}

        //mem::(&mut interpreter.environment, &mut self.1);
        // TODO: unclone
        // this is broken because
        //let initial_env = mem::replace(&mut interpreter.environment, self.1.clone());

        //interpreter.environment = self.1;
        let mut alt_environment = interpreter.environment.push();
        mem::swap(&mut alt_environment, &mut interpreter.environment);
        //let old_environment = interpreter.environment;
        //interpreter.environment = old_environment.push();
        for (param, arg) in self.definition.parameters.iter().zip(args) {
            interpreter.environment.define(&param.lexeme, arg);
        }

        let result = interpreter.eval(&self.definition.body);
        // pop function stack frame and closure environment
        //interpreter.environment.stack.truncate(initial_len);
        mem::swap(&mut alt_environment, &mut interpreter.environment);
        //interpreter.environment = old_environment;

        //interpreter.environment.pop();
        //interpreter.environment = initial_env;
        // TODO: update the callable env
        //mem::swap(&mut interpreter.environment, &mut self.1);
        match result {
            Ok(()) => Ok(Value::VNil),
            Err(RuntimeError::Return { value, .. }) => Ok(value),
            Err(err) => Err(err),
        }
    }
}
