use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use super::object_closure::ObjectClosure;
use super::object_instance::ObjectInstance;

#[derive(Clone)]
pub struct ObjectBoundMethod {
    pub method: Rc<ObjectClosure>,
    pub receiver: Rc<RefCell<ObjectInstance>>,
}

impl ObjectBoundMethod {
    pub fn new(receiver: Rc<RefCell<ObjectInstance>>, method: Rc<ObjectClosure>) -> Self {
        Self { method, receiver }
    }
}

impl Debug for ObjectBoundMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.method.function.fmt(f)
    }
}

impl PartialEq for ObjectBoundMethod {
    fn eq(&self, other: &Self) -> bool {
        self.receiver.as_ptr() == other.receiver.as_ptr() && Rc::as_ptr(&self.method) == Rc::as_ptr(&other.method)
    }
}
