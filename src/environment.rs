use crate::object::Object;
use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<&Object, String> {
        self.values
            .get(name)
            .ok_or(format!("Undefined variable {}.", name))
    }
}
