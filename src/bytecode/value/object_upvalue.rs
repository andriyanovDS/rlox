use std::ptr;
use super::Value;
use std::cmp::{Ordering, PartialEq, Ord, PartialOrd};
use std::fmt::{Debug, Formatter};

#[derive(Clone)]
pub struct ObjectUpvalue {
    location: *mut Value,
    closed: Option<Value>,
}

impl ObjectUpvalue {
    pub fn new(location: *mut Value) -> Self {
        Self {
            location,
            closed: None
        }
    }

    pub fn value(&self) -> &Value {
        unsafe {
            self.location.as_ref().unwrap()
        }
    }

    pub fn set_value(&self, value: Value) {
        unsafe {
            ptr::write(self.location, value);
        }
    }

    pub fn close_value(&mut self, value: Value) {
        self.closed = Some(value);
        self.location = self.closed.as_mut().unwrap();
    }
}

impl PartialEq for ObjectUpvalue {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location
    }
}

impl Eq for ObjectUpvalue {}

impl PartialOrd for ObjectUpvalue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.location.cmp(&other.location))
    }
}

impl Ord for ObjectUpvalue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.location.cmp(&other.location)
    }
}

impl Debug for ObjectUpvalue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Upvalue {:?}, is_closed: {}", self.value(), self.closed.is_some())
    }
}
