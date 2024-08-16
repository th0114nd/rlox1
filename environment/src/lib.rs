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

// Operations needed:
//   get: for each node in the linked list, attempt ot read from its map
//      &l, &k -> &v
//   define:  for the head of the linked list, insert into its map
//      &mut l.head, k, v -> ()
//   assign: for the first node that defines this key, update its value
//      &mut l.head, k, v -> ()
//          can be optimized with raw entry to take &k and convert to k via to_owned
//  fork: append a head to the linked list
//      &h, &t: the longer list should always outlive the shorter
//      may not be easy to expressive the recursive lifetimes

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
        //let raw_entry = self.table.raw_entry_mut().from_key(name);
        //match raw_entry {
        //    RawEntryMut::Occupied(mut o) => {
        //        o.insert(value.into());
        //    }
        //    RawEntryMut::Vacant(v) => {
        //        v.insert(name.to_owned(), value.into());
        //    }
        let mut last_map = self.stack.last_mut().unwrap();
        //.insert(name.to_owned(), value);
        let raw_entry = last_map.raw_entry_mut().from_key(name);
        match raw_entry {
            RawEntryMut::Occupied(mut o) => {
                o.insert(value.into());
            }
            RawEntryMut::Vacant(v) => {
                v.insert(name.to_owned(), value.into());
            }
        }
    }

    fn assign(&mut self, name: &str, value: Value) -> Result<(), ValueError> {
        for env in self.stack.iter_mut().rev() {
            let raw_entry = env.raw_entry_mut().from_key(name);
            //if let Some(v) = env.get_mut(name) {
            //    *v = value;
            //    return Ok(());
            //}
            match raw_entry {
                RawEntryMut::Occupied(mut o) => {
                    o.insert(value);
                    return Ok(());
                }
                RawEntryMut::Vacant(_) => continue,
                //    None => Err(ValueError::UndefinedVariable(name.to_string())),
                //    Some(t) => t.as_ref().assign(name, value),
                //},
            }
        }
        Err(ValueError::UndefinedVariable(name.to_owned()))
    }
}

//#[derive(Debug, Default, Clone)]
//pub struct Environment {
//    table: HashMap<String, Value>,
//    //parent: Option<Box<Environment>>,
//    //parent: Option<RefCell<Environment>>,
//}
//
//// fuck this
//#[derive(Debug, Default)]
//pub struct EnvList {
//    //head: Rc<RefCell<Environment>>,
//    //head: Rc<Cell<Environment>>,
//    head: Environment,
//    tail: Option<Box<EnvList>>,
//}

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

//impl Env for EnvList {
//    fn get(&self, name: &str) -> Result<&Value, ValueError> {
//        //let head_value = self.head.as_ref().get(name);
//        let head_value = self.head.get(name);
//        if head_value.is_ok() || self.tail.is_none() {
//            head_value
//        } else {
//            self.tail.as_ref().unwrap().get(name)
//            //self.tail.clone().unwrap().get(name)
//        }
//    }
//
//    fn define(&mut self, name: &str, value: Value) {
//        self.head.define(name, value)
//        //Rc::make_mut(&mut self.head).define(name, value)
//    }
//
//    #[allow(unused_variables)]
//    fn assign(&mut self, name: &str, value: Value) -> Result<(), ValueError> {
//        todo!()
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
//    }
//}

//impl Env for Environment {
//    fn get(&self, name: &str) -> Result<&Value, ValueError> {
//        self.table
//            .get(name)
//            .ok_or(ValueError::UndefinedVariable(name.to_owned()))
//    }
//
//    fn define(&mut self, name: &str, value: Value) {
//        let raw_entry = self.table.raw_entry_mut().from_key(name);
//        match raw_entry {
//            RawEntryMut::Occupied(mut o) => {
//                o.insert(value.into());
//            }
//            RawEntryMut::Vacant(v) => {
//                v.insert(name.to_owned(), value.into());
//            }
//        }
//    }
//
//    #[allow(unused_variables)]
//    fn assign(&mut self, name: &str, value: Value) -> Result<(), ValueError> {
//        todo!()
//        //    // may be a sign that we should be allocating string lexemes at scan time
//        //    let raw_entry = self.table.raw_entry_mut().from_key(name);
//        //    match raw_entry {
//        //        RawEntryMut::Occupied(mut o) => {
//        //            o.insert(value);
//        //            Ok(())
//        //        }
//        //        RawEntryMut::Vacant(_) => Err(ValueError::UndefinedVariable(name.to_string())),
//        //    }
//    }
//}

#[cfg(test)]
mod tests {
    use super::*;
    use models::Value::*;

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

        // TODO
        //env.fork();
        env.fork(|env| {
            //EnvList::fork(&mut env, |mut child_env| {
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
        //});
        //env.pop();

        // child update persisted
        assert_eq!(env.get("hello"), Ok(&Bool(false)));

        Ok(())
    }
}
