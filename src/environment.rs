//use crate::models::Value;
//use crate::models::ValueError;
use crate::value::Value;
use crate::value::ValueError;
use std::collections::hash_map::RawEntryMut;
use std::collections::HashMap;

pub trait Env {
    fn get(&self, name: &str) -> Result<&Value, ValueError>;
    fn define(&mut self, name: &str, value: Value);
    fn assign(&mut self, name: &str, value: Value) -> Result<(), ValueError>;
}

pub struct Environment {
    stack: Vec<HashMap<String, Value>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            stack: vec![HashMap::new()],
        }
    }
}

impl Environment {
    pub fn fork<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.stack.push(HashMap::new());
        let res = f(self);
        self.stack.pop();
        res
    }

    pub fn push(&mut self) {
        self.stack.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }
}

impl Env for Environment {
    fn get(&self, name: &str) -> Result<&Value, ValueError> {
        for env in self.stack.iter().rev() {
            if let Some(value) = env.get(name) {
                return Ok(value);
            }
        }
        Err(ValueError::UndefinedVariable(name.to_owned()))
    }

    fn define(&mut self, name: &str, value: Value) {
        let raw_entry = self
            .stack
            .last_mut()
            .unwrap()
            .raw_entry_mut()
            .from_key(name);
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
        for env in self.stack.iter_mut().rev() {
            let raw_entry = env.raw_entry_mut().from_key(name);
            match raw_entry {
                RawEntryMut::Occupied(mut o) => {
                    o.insert(value);
                    return Ok(());
                }
                RawEntryMut::Vacant(_) => continue,
            }
        }
        Err(ValueError::UndefinedVariable(name.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Value::*;

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
    }

    #[test]
    fn test_list() -> Result<(), ValueError> {
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

        env.fork(|env| {
            assert_eq!(env.get("hello"), Ok(&VString("world".to_owned())));
            // Overrides in parent
            env.assign("hello", Bool(false))?;
            assert_eq!(env.get("hello"), Ok(&Bool(false)));

            // Creates local def
            env.define("hello", VNumber(117.0));
            assert_eq!(env.get("hello"), Ok(&VNumber(117.0)));

            // Overrides locally
            env.assign("hello", VNumber(13.0))?;
            assert_eq!(env.get("hello"), Ok(&VNumber(13.0)));
            Ok::<(), ValueError>(())
        })?;

        // child update persisted
        assert_eq!(env.get("hello"), Ok(&Bool(false)));

        Ok(())
    }
}
