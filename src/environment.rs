use crate::object::Object;
use hash_map::Entry;
use std::cell::RefCell;
use std::collections::{hash_map, HashMap};
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

    pub fn get_at_distance(&self, distance: usize, name: &str) -> Result<Object, String> {
        if distance == 0 {
            return self.get(name);
        }
        let ancestor = self.ancestor(distance);
        let env = ancestor.as_ref().borrow();
        env.get(name)
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

    pub fn assign_at_distance(
        &mut self,
        distance: usize,
        name: String,
        value: Object,
    ) -> Result<(), String> {
        if distance == 0 {
            return self.assign(name, value);
        }
        let ancestor = self.ancestor(distance);
        let mut env = ancestor.as_ref().borrow_mut();
        env.assign(name, value)
    }

    fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment>> {
        let mut env = self.enclosing.as_ref().unwrap().clone();
        let mut depth = 1;
        while depth < distance {
            depth += 1;
            let env_ref = env.as_ref();
            let enclosing = env_ref.borrow().enclosing.as_ref().unwrap().clone();
            env = enclosing;
        }
        env
    }

    fn get_from_enclosing(&self, name: &str) -> Option<Result<Object, String>> {
        self.enclosing
            .as_ref()
            .map(|env| env.as_ref().borrow().get(name))
    }
}
