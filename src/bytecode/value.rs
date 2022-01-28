use std::fmt::{Debug};
use std::cmp::PartialEq;

#[derive(Clone, Debug)]
pub enum Value {
    Number(f32),
    Bool(bool),
    Nil,
    String {
        value: String,
        hash: usize,
    },
}

impl Value {
    pub fn make_string_value(string: String) -> Self {
        let hash = Value::hash_string(&string);
        Self::String {
            value: string,
            hash,
        }
    }

    pub fn hash_string(string: &str) -> usize {
        let bytes = string.as_bytes();
        bytes.into_iter().fold(0xcbf29ce484222325, |acc, byte| {
            (acc ^ (*byte as usize)).wrapping_mul(0x100000001b3)
        })
    }
}

impl Default for Value {
    fn default() -> Self { Self::Nil }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::Bool(left), Value::Bool(right)) => left == right,
            (Value::Nil, Value::Nil) => true,
            (Value::String { value: left, hash: _ }, Value::String { value: right, hash: _ }) => left == right,
            _ => false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(Value::hash_string("feedface"), 0x0a83c86fee952abc);
    }
}
