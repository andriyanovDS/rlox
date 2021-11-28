use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use super::callable::{Callable, LoxFn};
use super::object::Object;

pub const CONSTRUCTOR_KEYWORD: &'static str = "init";
pub const THIS_KEYWORD: &'static str = "this";
pub const SUPER_KEYWORD: &'static str = "super";

pub struct LoxClass {
    pub name: String,
    pub methods: HashMap<String, LoxFn>,
    pub static_methods: HashMap<String, LoxFn>,
    pub superclass: Option<Rc<LoxClass>>,
}

pub struct Instance {
    class: Rc<LoxClass>,
    fields: HashMap<String, Object>
}

impl LoxClass {
    pub fn find_method(&self, name: &str) -> Option<&LoxFn> {
        self.methods
            .get(name)
            .or_else(|| {
                self.superclass
                    .as_ref()
                    .and_then(|class| class.as_ref().find_method(name))
            })
    }

    pub fn find_static_method(&self, name: &str) -> Result<&LoxFn, String> {
        self.static_methods.get(name).ok_or_else(|| format!("Undefined static method {}.", name))
    }
}

impl Instance {
    pub fn new(class: Rc<LoxClass>) -> Self {
        Self { class, fields: HashMap::new() }
    }

    pub fn get(&self, name: &str, this: Rc<RefCell<Instance>>) -> Result<Object, String> {
        self.fields
            .get(name)
            .map(|v| v.clone())
            .or_else(|| self.find_method(name, this))
            .ok_or_else(|| format!("Undefined property {}.", name))
    }

    pub fn set(&mut self, name: String, value: Object) {
        self.fields.insert(name, value);
    }

    pub fn find_method(&self, name: &str, this: Rc<RefCell<Instance>>) -> Option<Object> {
        self.class.find_method(name).map(|func| Object::Callable(Callable::LoxFn(func.bind(this))))
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
