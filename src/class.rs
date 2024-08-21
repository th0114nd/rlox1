use compact_str::CompactString;
use std::fmt;

//pub trait AnyClass: fmt::Display + fmt::Debug {}
pub trait AnyClass: fmt::Display + fmt::Debug {}
//struct LoxClass {
//
//},h
//
#[derive(Debug)]
pub struct LoxClass {
    pub name: CompactString,
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl AnyClass for LoxClass {}
