use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use super::object_function::ObjectFunction;

pub struct ObjectClosure {
    pub function: Rc<ObjectFunction>
}

impl Debug for ObjectClosure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.function.fmt(f)
    }
}
