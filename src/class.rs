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

// Is every object callable? or just classes?
// I guess we could always panic?
pub trait AnyClass: fmt::Display + fmt::Debug + LoxCallable {
    fn get(&self, name: &Token) -> Result<Value, RuntimeError>;
    fn set(&self, name: &Token, value: Value);
}

#[derive(Debug, Clone)]
pub struct LoxClass {
    pub name: CompactString,
    pub methods: HashMap<CompactString, LoxFunction>,
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl LoxCallable for LoxClass {
    fn arity(&self) -> usize {
        // TODO: what is this actually?
        0
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let value = LoxInstance {
            class: self.clone(),
            fields: Default::default(),
        };
        // I know I know
        Ok(Value::Object(Rc::new(Rc::new(value))))
    }
}

impl AnyClass for LoxClass {
    fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        panic!("No get on class {self}.{}", name.lexeme);
    }
    fn set(&self, name: &Token, value: Value) {
        panic!("No set on class {self}.{} = {value}", name.lexeme);
    }
}

impl LoxClass {
    fn find_method(&self, name: &str) -> Option<LoxFunction> {
        self.methods.get(name).cloned()
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

impl LoxCallable for Rc<LoxInstance> {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        panic!("cannot call a class instance")
    }
}

impl AnyClass for Rc<LoxInstance> {
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
