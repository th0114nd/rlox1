use crate::error::LoxResult;
//use crate::stmt::StmtList;
use crate::stmt_eval::SEval;
use environment::Environment;
use models::StmtList;
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

impl<W: io::Write> Interpreter<W> {
    pub fn interpret(&mut self, stmts: StmtList) -> LoxResult<Vec<()>> {
        stmts
            .0
            .into_iter()
            .enumerate()
            .map(|(current, ref s)| s.eval(current + 1, &mut self.w, &mut self.environment))
            .collect()
    }
}
