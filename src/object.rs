use crate::callable::Callable;
use std::fmt;
use std::fmt::{Debug, Formatter};

#[derive(Debug, Clone)]
pub enum Object {
    Nil,
    Boolean(bool),
    String(String),
    Number(f64),
    Callable(Callable),
    Class(String),
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
            Object::Class(name) => write!(f, "{}", name)
        }
    }
}
