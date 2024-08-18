use crate::models::Value;
//use crate::models::RuntimeError;
use crate::error::RuntimeError;
use compact_str::CompactString;
use std::collections::hash_map::RawEntryMut;
use std::collections::HashMap;

pub trait Env {
    fn get(&self, name: &str) -> Result<&Value, RuntimeError>;
    fn define(&mut self, name: &str, value: Value);
    fn assign(&mut self, name: &str, value: Value) -> Result<(), RuntimeError>;
}

pub struct Environment {
    stack: Vec<HashMap<CompactString, Value>>,
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
    fn get(&self, name: &str) -> Result<&Value, RuntimeError> {
        for env in self.stack.iter().rev() {
            if let Some(value) = env.get(name) {
                return Ok(value);
            }
        }
        Err(RuntimeError::UndefinedVariable {
            line: "TODO".into(),
            name: name.into(),
        })
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
                v.insert(name.into(), value);
            }
        }
    }

    fn assign(&mut self, name: &str, value: Value) -> Result<(), RuntimeError> {
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
        Err(RuntimeError::UndefinedVariable {
            line: "TODO".into(),
            name: name.into(),
        })
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
        assert_eq!(env.get("hello"), Ok(&VNumber(95.3)));
        assert_eq!(env.assign("hello", Bool(true)), Ok(()));
        assert_eq!(env.get("hello"), Ok(&Bool(true)));

        env.define("hello", VString("world".into()));
        assert_eq!(env.get("hello"), Ok(&VString("world".into())));
    }

    #[test]
    fn test_list() -> Result<(), RuntimeError> {
        let mut env = Environment::default();

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
        assert_eq!(env.get("hello"), Ok(&VNumber(95.3)));
        assert_eq!(env.assign("hello", Bool(true)), Ok(()));
        assert_eq!(env.get("hello"), Ok(&Bool(true)));

        env.define("hello", VString("world".into()));
        assert_eq!(env.get("hello"), Ok(&VString("world".into())));

        env.fork(|env| {
            assert_eq!(env.get("hello"), Ok(&VString("world".into())));
            // Overrides in parent
            env.assign("hello", Bool(false))?;
            assert_eq!(env.get("hello"), Ok(&Bool(false)));

            // Creates local def
            env.define("hello", VNumber(117.0));
            assert_eq!(env.get("hello"), Ok(&VNumber(117.0)));

            // Overrides locally
            env.assign("hello", VNumber(13.0))?;
            assert_eq!(env.get("hello"), Ok(&VNumber(13.0)));
            Ok::<(), RuntimeError>(())
        })?;

        // child update persisted
        assert_eq!(env.get("hello"), Ok(&Bool(false)));

        Ok(())
    }
}
