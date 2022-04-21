use std::collections::HashMap;
use std::rc::Rc;
use crate::bytecode::value::Value;
use super::object_string::ObjectString;
use super::object_class::ObjectClass;

#[derive(Clone)]
pub struct ObjectInstance {
    pub class: Rc<ObjectClass>,
    fields: HashMap<Rc<ObjectString>, Value>
}

impl ObjectInstance {
    pub fn new(class: Rc<ObjectClass>) -> Self {
        Self { class, fields: HashMap::new() }
    }
}
