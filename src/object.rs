use crate::function::Callable;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Nil,
    Boolean(bool),
    String(String),
    Number(f64),
    Callable(Callable),
    NotInitialized,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Object::Nil => write!(f, "nil"),
            Object::Boolean(value) => write!(f, "{}", value),
            Object::String(value) => write!(f, "{}", value),
            Object::Number(value) => write!(f, "{}", value),
            Object::Callable(_) => write!(f, "{}", "function"),
            Object::NotInitialized => write!(f, "variable was not initialized"),
        }
    }
}
