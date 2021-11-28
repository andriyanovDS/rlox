use super::callable::Callable;
use super::lox_class::Instance;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::{self, Debug, Formatter};

#[derive(Debug, Clone)]
pub enum Object {
    Nil,
    Boolean(bool),
    String(String),
    Number(f64),
    Callable(Callable),
    Instance(Rc<RefCell<Instance>>),
    NotInitialized,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Object::Nil => write!(f, "nil"),
            Object::Boolean(value) => write!(f, "{}", value),
            Object::String(value) => write!(f, "{}", value),
            Object::Number(value) => write!(f, "{}", value),
            Object::Callable(callable) => callable.fmt(f),
            Object::NotInitialized => write!(f, "variable was not initialized"),
            Object::Instance(lox_class) => lox_class.as_ref().borrow().fmt(f),
        }
    }
}
