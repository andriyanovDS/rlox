use crate::object::Object;
use std::fmt::{Debug, Formatter};

#[derive(Clone)]
pub struct NativeFunction {
    pub arity: usize,
    pub on_call: Box<fn(&[Object]) -> Object>,
}

impl NativeFunction {
    pub fn call(&self, arguments: &[Object]) -> Object {
        (self.on_call)(arguments);
        Object::Nil
    }
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<function>")
    }
}
