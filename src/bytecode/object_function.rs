use std::cell::RefCell;
use std::rc::Rc;
use super::chunk::Chunk;
use super::object_string::ObjectString;

pub struct ObjectFunction {
    pub name: Rc<ObjectString>,
    pub arity: usize,
    pub chunk: Chunk,
}

pub enum FunctionType {
    Function(Rc<RefCell<ObjectFunction>>),
    Script(Chunk),
}