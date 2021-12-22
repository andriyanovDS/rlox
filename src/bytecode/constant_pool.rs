use super::vec::Vec;
use super::value::Value;

pub struct ConstantPool {
    values: Vec<Value>
}

impl ConstantPool {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn value(&self, index: usize) -> Value {
        self.values[index]
    }

    pub fn push(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn length(&self) -> usize {
        self.values.length
    }
}
