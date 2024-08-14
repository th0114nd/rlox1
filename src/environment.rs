use crate::value::Value;
use crate::value::ValueError;
use std::collections::hash_map::Entry;
use std::collections::hash_map::RawEntryMut;
use std::collections::HashMap;
use std::mem;

#[derive(Debug, Default)]
pub struct Environment(HashMap<String, Value>);

// TODO: reference count values so that we can avoid cloning every time
// we read them from the map
impl Environment {
    pub fn get(&self, name: &str) -> Result<Value, ValueError> {
        // TODO: should this need to clone?
        self.0
            .get(name)
            .ok_or(ValueError::UndefinedVariable(name.to_owned()))
            .cloned()
    }

    pub fn define(&mut self, name: &str, value: Value) {
        let raw_entry = self.0.raw_entry_mut().from_key(name);
        match raw_entry {
            RawEntryMut::Occupied(mut o) => {
                o.insert(value);
            }
            RawEntryMut::Vacant(v) => {
                v.insert(name.to_owned(), value);
            }
        }
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), ValueError> {
        // may be a sign that we should be allocating string lexemes at scan time
        let raw_entry = self.0.raw_entry_mut().from_key(name);
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
        assert_eq!(env.get("hello"), Ok(VNumber(95.3)));
        assert_eq!(env.assign("hello", Bool(true)), Ok(()));
        assert_eq!(env.get("hello"), Ok(Bool(true)));

        env.define("hello", VString("world".to_owned()));
        assert_eq!(env.get("hello"), Ok(VString("world".to_owned())));
    }
}
