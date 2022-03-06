use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use super::super::chunk::Chunk;
use super::object_string::ObjectString;

pub struct ObjectFunction {
    pub name: Rc<ObjectString>,
    pub arity: u8,
    pub chunk: Chunk,
}

impl Debug for ObjectFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Function {:?}, arity {}", self.name, self.arity)
    }
}
