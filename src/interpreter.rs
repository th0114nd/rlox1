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

impl<'a, W: io::Write> Interpreter<W> {
    pub fn interpret(&mut self, stmts: StmtList<'a>) -> LoxResult<Vec<()>> {
        stmts
            .0
            .into_iter()
            .enumerate()
            .map(|(current, ref s)| s.eval(current + 1, &mut self.w, &mut self.environment))
            .collect()
    }

    //pub rn
    //pub fn eval(self, mut w: impl io::Write) -> Result<Vec<()>, LoxError> {
    //    // TODO: env should outlive an evaluation, for example in an interpreter
    //    let mut env = Environment::default();
    //    self.0
    //        .into_iter()
    //        .enumerate()
    //        .map(|(current, ref s)| s.eval(current + 1, &mut w, &mut env))
    //        .collect::<Result<Vec<()>, LoxError>>()
    //}
}
