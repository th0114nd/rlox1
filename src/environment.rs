use crate::value::Value;
use crate::value::ValueError;
use std::collections::hash_map::RawEntryMut;
use std::collections::HashMap;

pub trait Env {
    fn get(&self, name: &str) -> Result<&Value, ValueError>;
    fn define(&mut self, name: &str, value: Value);
    fn assign(&mut self, name: &str, value: Value) -> Result<(), ValueError>;
}

#[derive(Debug, Default)]
pub struct Environment {
    table: HashMap<String, Value>,
    //parent: Option<Box<Environment>>,
    //parent: Option<RefCell<Environment>>,
}

#[derive(Debug, Default)]
pub struct EnvList {
    head: Environment,
    tail: Option<Box<Environment>>,
}

impl Env for EnvList {
    fn get(&self, name: &str) -> Result<&Value, ValueError> {
        let head_value = self.head.get(name);
        if head_value.is_ok() || self.tail.is_none() {
            head_value
        } else {
            self.tail.as_ref().unwrap().get(name)
        }
    }

    fn define(&mut self, name: &str, value: Value) {
        self.head.define(name, value)
    }

    fn assign(&mut self, name: &str, value: Value) -> Result<(), ValueError> {
        let raw_entry = self.head.table.raw_entry_mut().from_key(name);
        match raw_entry {
            RawEntryMut::Occupied(mut o) => {
                o.insert(value);
                Ok(())
            }
            RawEntryMut::Vacant(_) => match self.tail.as_mut() {
                None => Err(ValueError::UndefinedVariable(name.to_string())),
                Some(t) => t.assign(name, value),
            },
        }
    }
}

impl Env for Environment {
    fn get(&self, name: &str) -> Result<&Value, ValueError> {
        self.table
            .get(name)
            .ok_or(ValueError::UndefinedVariable(name.to_owned()))
    }

    fn define(&mut self, name: &str, value: Value) {
        let raw_entry = self.table.raw_entry_mut().from_key(name);
        match raw_entry {
            RawEntryMut::Occupied(mut o) => {
                o.insert(value);
            }
            RawEntryMut::Vacant(v) => {
                v.insert(name.to_owned(), value);
            }
        }
    }

    fn assign(&mut self, name: &str, value: Value) -> Result<(), ValueError> {
        // may be a sign that we should be allocating string lexemes at scan time
        let raw_entry = self.table.raw_entry_mut().from_key(name);
        match raw_entry {
            RawEntryMut::Occupied(mut o) => {
                o.insert(value);
                Ok(())
            }
            RawEntryMut::Vacant(_) => Err(ValueError::UndefinedVariable(name.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value::*;

    #[test]
    fn test_basic() {
        let mut env = Environment::default();

        assert_eq!(
            env.get("hello"),
            Err(ValueError::UndefinedVariable("hello".to_owned())),
        );

        assert_eq!(
            env.assign("hello", Bool(true)),
            Err(ValueError::UndefinedVariable("hello".to_owned())),
        );

        env.define("hello", VNumber(95.3));
        assert_eq!(env.get("hello"), Ok(&VNumber(95.3)));
        assert_eq!(env.assign("hello", Bool(true)), Ok(()));
        assert_eq!(env.get("hello"), Ok(&Bool(true)));

        env.define("hello", VString("world".to_owned()));
        assert_eq!(env.get("hello"), Ok(&VString("world".to_owned())));

        // TODO
        //env.fork(|mut child_env| {
        //    assert_eq!(child_env.get("hello"), Ok(&VString("world".to_owned())));
        //    // Overrides in parent
        //    child_env.assign("hello", Bool(false));
        //    assert_eq!(child_env.get("hello"), Ok(&Bool(false)));
        //
        //    // Creates local def
        //    child_env.define("hello", VNumber(117.0));
        //    assert_eq!(child_env.get("hello"), Ok(&VNumber(117.0)));
        //
        //    // Overrides locally
        //    child_env.assign("hello", VNumber(13.0));
        //    assert_eq!(child_env.get("hello"), Ok(&VNumber(13.0)));
        //});
        //
        //// child update persisted
        //assert_eq!(env.get("hello"), Ok(&Bool(false)));
    }
}
