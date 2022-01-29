use std::fmt::{Debug, Formatter};
use std::cmp::PartialEq;
use std::rc::Rc;
use super::object_string::ObjectString;

#[derive(Clone)]
pub enum Value {
    Number(f32),
    Bool(bool),
    Nil,
    String(Rc<ObjectString>),
}

impl Default for Value {
    fn default() -> Self { Self::Nil }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::Bool(left), Value::Bool(right)) => left == right,
            (Value::Nil, Value::Nil) => true,
            (Value::String(left), Value::String(right)) => {
                Rc::as_ptr(left) == Rc::as_ptr(right)
            },
            _ => false
        }
    }
}

impl Debug for Value {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(boolean) => write!(formatter, "{}", boolean),
            Value::Number(number) => write!(formatter, "{}", number),
            Value::String(object) => write!(formatter, "{:?}", object.as_ref().value),
            Value::Nil => write!(formatter, "Nil")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(ObjectString::hash_string("feedface"), 0x0a83c86fee952abc);
    }
}
