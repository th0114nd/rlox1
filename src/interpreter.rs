use crate::environment::Environment;
use crate::error::LoxResult;
use crate::models::StmtList;
use std::io;

#[derive(Default)]
pub struct Interpreter {
    pub environment: Environment,
    pub buffer: Vec<u8>,
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
    pub fn interpret(&mut self, stmts: StmtList) -> LoxResult<()> {
        stmts.0.into_iter().try_for_each(|ref s| self.eval(s))
    }
}
