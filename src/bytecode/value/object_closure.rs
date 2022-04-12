use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use super::object_upvalue::ObjectUpvalue;
use super::object_function::ObjectFunction;

pub struct ObjectClosure {
    pub function: Rc<ObjectFunction>,
    pub upvalues: Vec<Rc<RefCell<ObjectUpvalue>>>,
}

impl Debug for ObjectClosure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.function.fmt(f)
    }
}
