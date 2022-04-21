use std::rc::Rc;
use super::object_string::ObjectString;

#[derive(Clone)]
pub struct ObjectClass {
    pub name: Rc<ObjectString>,
}

impl ObjectClass {
    pub fn new(name: Rc<ObjectString>) -> Self {
        Self { name }
    }
}
