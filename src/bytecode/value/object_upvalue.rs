use std::ptr;
use super::Value;

#[derive(Copy, Clone)]
pub struct ObjectUpvalue {
    location: *mut Value,
}

impl ObjectUpvalue {
    pub fn new(location: *mut Value) -> Self {
        Self { location }
    }

    pub fn location(&self) -> &Value {
        unsafe {
            self.location.as_ref().unwrap()
        }
    }

    pub fn set(&self, value: Value) {
        unsafe {
            ptr::write(self.location, value);
        }
    }
}
