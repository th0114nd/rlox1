use crate::callable::LoxCallable;
use crate::callable::LoxFunction;
use crate::error::RuntimeError;
use crate::interpreter::Interpreter;
use crate::models::Token;
use crate::models::Value;
use compact_str::CompactString;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct LoxClass {
    pub name: CompactString,
    pub methods: HashMap<CompactString, LoxFunction>,
    pub parent: Option<Rc<LoxClass>>,
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl LoxCallable for LoxClass {
    fn arity(&self) -> usize {
        match self.find_method("init") {
            None => 0,
            Some(f) => f.arity(),
        }
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let instance = Rc::new(LoxInstance {
            class: self.clone(),
            fields: Default::default(),
        });
        if let Some(init) = self.find_method("init") {
            init.bind(instance.clone()).call(interpreter, args)?;
        }
        Ok(Value::Object(instance))
    }
}

impl LoxClass {
    pub fn find_method(&self, name: &str) -> Option<LoxFunction> {
        self.methods
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.find_method(name)))
    }
}

#[derive(Debug)]
pub struct LoxInstance {
    // Clone for now, but this should maybe be refcounted
    class: LoxClass,

    fields: RefCell<HashMap<CompactString, Value>>,
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} instance", self.class)
    }
}

pub trait GetSet {
    fn get(&self, name: &Token) -> Result<Value, RuntimeError>;
    fn set(&self, name: &Token, value: Value);
}

impl GetSet for Rc<LoxInstance> {
    fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        let borrow = self.fields.borrow();
        if let Some(value) = borrow.get(&name.lexeme) {
            return Ok(value.clone());
        }
        if let Some(method) = self.class.find_method(&name.lexeme) {
            let x: Rc<LoxInstance> = self.clone();
            return Ok(Value::Callable(Rc::new(method.bind(x))));
        }
        Err(RuntimeError::UndefinedProperty {
            line: "TODO".into(),
            name: name.lexeme.to_owned(),
        })
    }

    fn set(&self, name: &Token, value: Value) {
        let mut borrow = self.fields.borrow_mut();
        borrow.insert(name.lexeme.to_owned(), value);
    }
}
