use crate::interpreter::Interpreter;
use crate::object::Object;
use std::fmt::{Debug, Formatter};

#[derive(Clone)]
pub struct Callable {
    pub arity: usize,
    pub on_call: Box<fn(&[Object]) -> Object>
}

impl Callable {
    pub fn call(&self, interpreter: &Interpreter, arguments: &[Object]) -> Object {
        todo!()
    }
}

impl Debug for Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<function>")
    }
}
