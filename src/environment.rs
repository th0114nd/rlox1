use crate::value::Value;
use crate::value::ValueError;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Environment(HashMap<String, Value>);

impl Environment {
    pub fn get(&self, name: &str) -> Result<Value, ValueError> {
        // TODO: should this need to clone?
        self.0
            .get(name)
            .ok_or(ValueError::UndefinedVariable(name.to_owned()))
            .cloned()
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.0.insert(name.to_string(), value);
    }
}
