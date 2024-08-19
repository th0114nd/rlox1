#![feature(hash_raw_entry)]
use compact_str::CompactString;
use std::cell::RefCell;
use std::collections::hash_map::RawEntryMut;
use std::collections::HashMap;
use std::rc::Rc;
use thiserror::Error;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Value {
    #[default]
    VNil,
    Bool(bool),
    VNumber(f64),
    VString(CompactString),
}

#[derive(Debug, Error, PartialEq)]
pub enum RuntimeError {
    #[error("[line {line}] Error: undefined variable: '{name}'")]
    UndefinedVariable {
        line: CompactString,
        name: CompactString,
    },
}

pub trait Env {
    fn get(&self, name: &str) -> Result<Value, RuntimeError>;
    fn define(&self, name: &str, value: Value);
    fn assign(&self, name: &str, value: Value) -> Result<(), RuntimeError>;
}

//#[derive(Debug, Clone)]
//pub struct Environment {
//    // TODO: hide when the API works for function calls
//    pub stack: Vec<RefCell<HashMap<CompactString, Value>>>,
//}

//impl Default for Environment {
//    fn default() -> Self {
//        Self {
//            stack: vec![HashMap::new().into()],
//        }
//    }
//}

#[derive(Default)]
struct EnvList {
    table: RefCell<HashMap<CompactString, Value>>,
    parent: Option<Rc<EnvList>>,
}

impl EnvList {
    fn push(self: &Rc<Self>) -> Rc<Self> {
        Rc::new(Self {
            table: HashMap::new().into(),
            parent: Some(self.clone()),
        })
    }

    fn pop(mut self: Rc<Self>) -> Option<Rc<Self>> {
        // this is a silly method -- we shouldn't need to cloen
        // to move out
        self.parent.clone()
        //use std::mem;
        //mem::replace(&mut self.parent, None)
    }
}

impl Env for EnvList {
    fn get(&self, name: &str) -> Result<Value, RuntimeError> {
        {
            let borrow = self.table.borrow();
            if let Some(value) = borrow.get(name) {
                return Ok(value.clone());
            }
        }
        match &self.parent {
            None => Err(RuntimeError::UndefinedVariable {
                line: "TODO".into(),
                name: name.into(),
            }),
            Some(env) => env.get(name),
        }
    }

    fn define(&self, name: &str, value: Value) {
        let mut borrow = self.table.borrow_mut();
        let raw_entry = borrow.raw_entry_mut().from_key(name);
        match raw_entry {
            RawEntryMut::Occupied(mut o) => {
                o.insert(value);
            }
            RawEntryMut::Vacant(v) => {
                v.insert(name.into(), value);
            }
        }
    }

    fn assign(&self, name: &str, value: Value) -> Result<(), RuntimeError> {
        {
            let mut borrow = self.table.borrow_mut();
            let raw_entry = borrow.raw_entry_mut().from_key(name);
            if let RawEntryMut::Occupied(mut o) = raw_entry {
                o.insert(value);
                return Ok(());
            }
        }
        match &self.parent {
            None => Err(RuntimeError::UndefinedVariable {
                line: "TODO".into(),
                name: name.into(),
            }),
            Some(env) => env.assign(name, value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Value::*;
    use super::*;

    #[test]
    fn test_basic() {
        let env = EnvList::default();

        assert_eq!(
            env.get("hello"),
            Err(RuntimeError::UndefinedVariable {
                line: "TODO".into(),
                name: "hello".into()
            }),
        );

        assert_eq!(
            env.assign("hello", Bool(true)),
            Err(RuntimeError::UndefinedVariable {
                line: "TODO".into(),
                name: "hello".into()
            }),
        );

        env.define("hello", VNumber(95.3));
        assert_eq!(env.get("hello"), Ok(VNumber(95.3)));
        assert_eq!(env.assign("hello", Bool(true)), Ok(()));
        assert_eq!(env.get("hello"), Ok(Bool(true)));

        env.define("hello", VString("world".into()));
        assert_eq!(env.get("hello"), Ok(VString("world".into())));
    }

    #[test]
    fn test_list() -> Result<(), RuntimeError> {
        let env = Rc::new(EnvList::default());

        assert_eq!(
            env.get("hello"),
            Err(RuntimeError::UndefinedVariable {
                line: "TODO".into(),
                name: "hello".into()
            }),
        );

        assert_eq!(
            env.assign("hello", Bool(true)),
            Err(RuntimeError::UndefinedVariable {
                line: "TODO".into(),
                name: "hello".into()
            }),
        );

        env.define("hello", VNumber(95.3));
        assert_eq!(env.get("hello"), Ok(VNumber(95.3)));
        assert_eq!(env.assign("hello", Bool(true)), Ok(()));
        assert_eq!(env.get("hello"), Ok(Bool(true)));

        env.define("hello", VString("world".into()));
        assert_eq!(env.get("hello"), Ok(VString("world".into())));

        //env.fork(|env| {
        let env = env.push();
        assert_eq!(env.get("hello"), Ok(VString("world".into())));
        // Overrides in parent
        env.assign("hello", Bool(false))?;
        assert_eq!(env.get("hello"), Ok(Bool(false)));

        // Creates local def
        env.define("hello", VNumber(117.0));
        assert_eq!(env.get("hello"), Ok(VNumber(117.0)));

        // Overrides locally
        env.assign("hello", VNumber(13.0))?;
        assert_eq!(env.get("hello"), Ok(VNumber(13.0)));
        //Ok::<(), RuntimeError>(())
        //})?;
        let env = env.pop().expect("must be able to pop");

        // child update persisted
        assert_eq!(env.get("hello"), Ok(Bool(false)));

        Ok(())
    }

    #[test]
    fn test_closure() -> Result<(), RuntimeError> {
        let env = Rc::new(EnvList::default());
        env.define("global", VNumber(420.0));

        let closure1 = env.push();
        let closure2 = env.push();
        closure1.define("i", VNumber(10.0));
        closure2.get("i").expect_err("should have error");
        env.get("i").expect_err("should have error");
        assert_eq!(closure1.get("i"), Ok(VNumber(10.0)));
        Ok(())
    }
}
