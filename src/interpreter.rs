use crate::callable::Clock;
use crate::environment::Env;
use crate::environment::Environment;
use crate::error::RuntimeError;
use crate::models::Expr;
use crate::models::StmtList;
use crate::models::Value;
use std::collections::HashMap;
use std::io;
use std::rc::Rc;

pub struct Interpreter {
    pub globals: Rc<Environment>,
    pub environment: Rc<Environment>,
    pub buffer: Vec<u8>,
    pub resolutions: HashMap<*const Expr, usize>,
}

impl Default for Interpreter {
    fn default() -> Self {
        let clock = Rc::new(Clock);
        let mut def = Self {
            globals: Default::default(),
            environment: Default::default(),
            buffer: Default::default(),
            resolutions: Default::default(),
        };
        def.environment = def.globals.clone();
        def.environment.define("clock", Value::Callable(clock));
        def
    }
}
#[cfg(test)]
impl io::Write for Interpreter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buffer.flush()
    }
}

#[cfg(not(test))]
impl io::Write for Interpreter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        io::stdout().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()
    }
}

impl Interpreter {
    pub fn interpret(&mut self, stmts: &StmtList) -> Result<(), RuntimeError> {
        stmts.into_iter().try_for_each(|s| self.eval(s))
    }
}
