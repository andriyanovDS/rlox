use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use crate::callable::Callable;
use crate::object::Object;

pub struct LoxClass {
    pub name: String,
    pub methods: HashMap<String, Callable>
}

pub struct Instance {
    class: Rc<LoxClass>,
    fields: HashMap<String, Object>
}

impl LoxClass {

}

impl Instance {
    pub fn new(class: Rc<LoxClass>) -> Self {
        Self { class, fields: HashMap::new() }
    }

    pub fn get(&self, name: &str) -> Result<Object, String> {
        self.fields
            .get(name)
            .map(|v| v.clone())
            .or_else(|| self.find_method(name))
            .ok_or_else(|| format!("Undefined property {}.", name))
    }

    pub fn set(&mut self, name: String, value: Object) {
        self.fields.insert(name, value);
    }

    fn find_method(&self, name: &str) -> Option<Object> {
        self.class.methods
            .get(name)
            .map(|func| Object::Callable(func.clone()))
    }
}

impl Debug for LoxClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Debug for Instance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}
