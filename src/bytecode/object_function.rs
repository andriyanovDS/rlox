use std::rc::Rc;
use super::chunk::Chunk;
use super::object_string::ObjectString;

pub struct ObjectFunction {
    pub name: Rc<ObjectString>,
    pub arity: u8,
    pub chunk: Chunk,
}