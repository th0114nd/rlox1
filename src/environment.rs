use crate::error::RuntimeError;
use crate::models::Value;
use compact_str::CompactString;
use std::cell::RefCell;
use std::collections::hash_map::RawEntryMut;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

pub trait Env {
    fn get(&self, name: &str) -> Result<Value, RuntimeError>;
    fn get_at(&self, name: &str, depth: usize) -> Result<Value, RuntimeError>;
    fn define(&self, name: &str, value: Value);
    fn assign(&self, name: &str, value: Value) -> Result<(), RuntimeError>;
    fn assign_at(&self, name: &str, value: Value, depth: usize) -> Result<(), RuntimeError>;
}

#[derive(Default, Debug)]
pub struct Environment {
    table: RefCell<HashMap<CompactString, Value>>,
    parent: Option<Rc<Environment>>,
}

impl Environment {
    pub fn push(self: &Rc<Self>) -> Rc<Self> {
        Rc::new(Self {
            table: HashMap::new().into(),
            parent: Some(self.clone()),
        })
    }

    fn pop(self: Rc<Self>) -> Option<Rc<Self>> {
        // this is a silly method -- we shouldn't need to clone
        // to move out. But self is Rc and so immutable
        self.parent.clone()
    }
}

impl fmt::Display for Environment {
    fn fmt(self: &Environment, f: &mut fmt::Formatter) -> fmt::Result {
        let mut depth = 0;
        let mut env = self;
        loop {
            writeln!(f, "{depth}: {:?}", env.table.borrow())?;
            depth += 1;
            match &env.parent {
                None => return Ok(()),
                Some(penv) => env = penv,
            }
        }
    }
}

impl Env for Environment {
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

    fn get_at(&self, name: &str, depth: usize) -> Result<Value, RuntimeError> {
        if depth == 0 {
            self.get(name)
        } else {
            self.parent
                .as_ref()
                .expect("we checked parent depth earlier")
                .get_at(name, depth - 1)
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

    fn assign_at(&self, name: &str, value: Value, depth: usize) -> Result<(), RuntimeError> {
        if depth == 0 {
            self.assign(name, value)
        } else {
            self.assign_at(name, value, depth - 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Value::*;

    #[test]
    fn test_basic() {
        let env = Environment::default();

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
        let env = Rc::new(Environment::default());

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
        let env = Rc::new(Environment::default());
        env.define("global", VNumber(420.0));

        let closure1 = env.push();
        let closure2 = env.push();
        closure1.define("i", VNumber(10.0));
        closure2.get("i").expect_err("should have error");
        env.get("i").expect_err("should have error");
        assert_eq!(closure1.get("i"), Ok(VNumber(10.0)));
        Ok(())
    }

    #[test]
    fn test_addressing() -> Result<(), RuntimeError> {
        let base = Rc::new(Environment::default());
        base.define("name", VString("base".into()));
        let mid = base.push();
        mid.define("name", VString("mid".into()));
        let top = mid.push();
        top.define("name", VString("top".into()));

        assert_eq!(top.get("name")?, VString("top".into()));
        assert_eq!(top.get_at("name", 0)?, VString("top".into()));
        assert_eq!(top.get_at("name", 1)?, VString("mid".into()));
        assert_eq!(top.get_at("name", 2)?, VString("base".into()));
        Ok(())
    }
}
