use std::collections::HashMap;
use std::rc::Rc;
use super::object_closure::ObjectClosure;
use super::object_string::ObjectString;

#[derive(Clone)]
pub struct ObjectClass {
    pub name: Rc<ObjectString>,
    methods: HashMap<Rc<ObjectString>, Rc<ObjectClosure>>,
}

impl ObjectClass {
    pub fn new(name: Rc<ObjectString>) -> Self {
        Self {
            name,
            methods: HashMap::new()
        }
    }

    pub fn add_method(&mut self, name: Rc<ObjectString>, method: Rc<ObjectClosure>) {
        self.methods.insert(name, method);
    }

    pub fn method(&self, name: &Rc<ObjectString>) -> Option<&Rc<ObjectClosure>> {
        self.methods.get(name)
    }
}
