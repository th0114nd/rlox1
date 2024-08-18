use crate::error::LoxResult;
//use crate::stmt::StmtList;
//use crate::stmt_eval::SEval;
use crate::environment::Environment;
use crate::models::StmtList;
use std::io;
use std::rc::Rc;

#[derive(Default)]
pub struct Interpreter {
    pub environment: Environment,
    pub buffer: Vec<u8>, //cfg!(te
                         //    pub w: Vec<u8>,
                         //} else {
                         //    pub w: io::Stdout,
                         //}
                         //pub w: W,
}

impl Interpreter {
    //#[cfg(test)]
    fn writer(&mut self) -> &dyn io::Write {
        // TODO: this is broken
        //Box::new(self.buffer.clone())
        &self.buffer
    }

    //#[cfg(not(test))]
    //fn writer(&mut self) -> Box<dyn io::Write> {
    //    io::stdout()
    //}
}

//#[cfg(test)]
//impl io::Write for Interpreter {
//    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
//        self.buffer.write(buf)
//    }
//
//    fn flush(&mut self) -> io::Result<()> {
//        self.buffer.flush()
//    }
//}
//
//#[cfg(not(test))]
//impl io::Write for Interpreter {
//    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
//        io::stdout().write(buf)
//    }
//
//    fn flush(&mut self) -> io::Result<()> {
//        io::stdout().flush()
//    }
//}

//impl Default for Interpreter<io::Stdout> {
//    fn default() -> Self {
//        Interpreter {
//            environment: Default::default(),
//            w: io::stdout(),
//        }
//    }
//}
//
//impl Default for Interpreter<Vec<u8>> {
//    fn default() -> Self {
//        Interpreter {
//            environment: Default::default(),
//            w: Default::default(),
//        }
//    }
//}

impl Interpreter {
    pub fn interpret(&mut self, stmts: StmtList) -> LoxResult<()> {
        stmts.0.into_iter().try_for_each(|ref s| self.eval(s))
    }
}
