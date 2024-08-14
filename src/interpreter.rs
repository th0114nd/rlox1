use crate::environment::Environment;
use crate::error::LoxResult;
use crate::stmt::StmtList;
use std::io;

pub struct Interpreter<W: io::Write> {
    pub environment: Environment,
    pub w: W,
}

impl Default for Interpreter<io::Stdout> {
    fn default() -> Self {
        Interpreter {
            environment: Default::default(),
            w: io::stdout(),
        }
    }
}

impl Default for Interpreter<Vec<u8>> {
    fn default() -> Self {
        Interpreter {
            environment: Default::default(),
            w: Default::default(),
        }
    }
}

impl<'a, W: io::Write> Interpreter<W> {
    pub fn interpret(&mut self, stmts: StmtList<'a>) -> LoxResult<Vec<()>> {
        stmts
            .0
            .into_iter()
            .enumerate()
            .map(|(current, ref s)| s.eval(current + 1, &mut self.w, &mut self.environment))
            .collect()
    }
}
