use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::bytecode::value::Value;
use super::object_string::ObjectString;
use super::object_class::ObjectClass;

#[derive(Clone)]
pub struct ObjectInstance {
    pub class: Rc<RefCell<ObjectClass>>,
    fields: HashMap<Rc<ObjectString>, Value>
}

impl ObjectInstance {
    pub fn new(class: Rc<RefCell<ObjectClass>>) -> Self {
        Self { class, fields: HashMap::new() }
    }

    pub fn property(&self, name: &Rc<ObjectString>) -> Option<&Value> {
        self.fields.get(name)
    }

    pub fn set_property(&mut self, name: Rc<ObjectString>, value: Value) {
        self.fields.insert(name, value);
    }
}
