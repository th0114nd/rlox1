#![feature(hash_raw_entry)]
//use crate::value::Value;
//use crate::value::ValueError;
//use std::borrow::BorrowMut;
//use std::cell::Cell;
use models::Value;
use models::ValueError;
//use std::cell::RefCell;
use std::collections::hash_map::RawEntryMut;
use std::collections::HashMap;
//use std::rc::Rc;

//#[derive(Debug)]
//struct Value;
//#[derive(Debug)]
//struct ValueError;
//
pub trait Env {
    //fn get(&self, name: &str) -> Result<Rc<Value>, ValueError>;
    fn get(&self, name: &str) -> Result<&Value, ValueError>;
    fn define(&mut self, name: &str, value: Value);
    fn assign(&mut self, name: &str, value: Value) -> Result<(), ValueError>;
}

#[derive(Debug, Default, Clone)]
pub struct Environment {
    table: HashMap<String, Value>,
    //parent: Option<Box<Environment>>,
    //parent: Option<RefCell<Environment>>,
}

#[derive(Debug, Default)]
pub struct EnvList {
    //head: Rc<RefCell<Environment>>,
    //head: Rc<Cell<Environment>>,
    head: Environment,
    tail: Option<Box<EnvList>>,
}

//impl EnvList {
//    fn fork(this: &mut Rc<RefCell<EnvList>>, f: impl FnOnce(EnvList)) {
//        //let new_head = Rc::new(RefCell::new(Environment::default()));
//        //let mut new_list = EnvList::default();
//        //new_list.tail = Some(this.clone());
//        //// TODO: no clone
//        //// but what a headache
//        //let new_tail = Some(Box::new(self.clone()));
//        //let new_list = EnvList {
//        //    head: new_head,
//        //    tail: new_tail,
//        //};
//        f(new_list)
//    }
//}

//impl Env for EnvList {
//    fn get(&self, name: &str) -> Result<&Value, ValueError> {
//        self.head.get(name)
//    }
//    fn define(&mut self, name: &str, value: Value) {
//        //*self.define(name, value)
//        self.head.define(name, value)
//    }
//    #[allow(unused_variables)]
//    fn assign(&mut self, name: &str, value: Value) -> Result<(), ValueError> {
//        todo!()
//    }
//}

impl Env for EnvList {
    fn get(&self, name: &str) -> Result<&Value, ValueError> {
        //let head_value = self.head.as_ref().get(name);
        let head_value = self.head.get(name);
        if head_value.is_ok() || self.tail.is_none() {
            head_value
        } else {
            self.tail.as_ref().unwrap().get(name)
            //self.tail.clone().unwrap().get(name)
        }
    }

    fn define(&mut self, name: &str, value: Value) {
        self.head.define(name, value)
        //Rc::make_mut(&mut self.head).define(name, value)
    }

    #[allow(unused_variables)]
    fn assign(&mut self, name: &str, value: Value) -> Result<(), ValueError> {
        todo!()
        ////let head = self.head.assume_init();
        //let head = Rc::make_mut(&mut self.head);
        //let raw_entry = head.table.raw_entry_mut().from_key(name);
        //match raw_entry {
        //    RawEntryMut::Occupied(mut o) => {
        //        o.insert(value);
        //        Ok(())
        //    }
        //    RawEntryMut::Vacant(_) => match self.tail.as_mut() {
        //        None => Err(ValueError::UndefinedVariable(name.to_string())),
        //        Some(t) => t.as_ref().assign(name, value),
        //    },
        //}
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
                o.insert(value.into());
            }
            RawEntryMut::Vacant(v) => {
                v.insert(name.to_owned(), value.into());
            }
        }
    }

    #[allow(unused_variables)]
    fn assign(&mut self, name: &str, value: Value) -> Result<(), ValueError> {
        todo!()
        //    // may be a sign that we should be allocating string lexemes at scan time
        //    let raw_entry = self.table.raw_entry_mut().from_key(name);
        //    match raw_entry {
        //        RawEntryMut::Occupied(mut o) => {
        //            o.insert(value);
        //            Ok(())
        //        }
        //        RawEntryMut::Vacant(_) => Err(ValueError::UndefinedVariable(name.to_string())),
        //    }
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
    }

    #[test]
    fn test_list() {
        let mut env = EnvList::default();

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
        //EnvList::fork(&mut env, |mut child_env| {
        //    assert_eq!(
        //        child_env.get("hello"),
        //        Ok(VString("world".to_owned()).into())
        //    );
        //    // Overrides in parent
        //    child_env.assign("hello", Bool(false));
        //    assert_eq!(child_env.get("hello"), Ok(Bool(false).into()));
        //
        //    // Creates local def
        //    child_env.define("hello", VNumber(117.0));
        //    assert_eq!(child_env.get("hello"), Ok(VNumber(117.0).into()));
        //
        //    // Overrides locally
        //    child_env.assign("hello", VNumber(13.0));
        //    assert_eq!(child_env.get("hello"), Ok(VNumber(13.0).into()));
        //});
        //
        //// child update persisted
        //assert_eq!(env.get("hello"), Ok(Bool(false).into()));
    }
}
