use crate::object::Object;
use std::cell::RefCell;
use std::collections::{HashMap, hash_map};
use hash_map::Entry;
use std::rc::Rc;

pub struct Environment {
    values: HashMap<String, Object>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn from(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<Object, String> {
        self.values
            .get(name)
            .map(|obj| Ok(obj.clone()))
            .or_else(|| self.get_from_enclosing(name))
            .unwrap_or_else(|| Err(format!("Undefined variable {}.", name)))
            .and_then(|obj| match obj {
                Object::NotInitialized => {
                    Err(format!("Variable {} must be initialized before use.", name))
                }
                _ => Ok(obj),
            })
    }

    pub fn assign(&mut self, name: String, value: Object) -> Result<(), String> {
        if let Entry::Occupied(mut entry) = self.values.entry(name.clone()) {
            entry.insert(value);
            return Ok(());
        }
        let enclosing = self.enclosing.as_ref().map(|env| env.as_ref().borrow_mut());
        match enclosing {
            Some(mut enclosing) => enclosing.assign(name, value),
            None => Err(format!("Undefined variable {}.", name)),
        }
    }

    fn get_from_enclosing(&self, name: &str) -> Option<Result<Object, String>> {
        self.enclosing
            .as_ref()
            .map(|env| env.as_ref().borrow().get(name))
    }
}
