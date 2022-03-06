use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use super::super::hash_table::{Hashable, HashTable};

#[derive(Clone)]
pub struct ObjectString {
    pub value: String,
    hash: usize,
}

impl Debug for ObjectString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl ObjectString {
    pub fn from_string(string: String) -> Self {
        let hash = ObjectString::hash_string(&string);
        Self {
            value: string,
            hash,
        }
    }

    pub fn new(string: String, hash: usize) -> Self {
        Self { value: string, hash }
    }

    pub fn hash_string(string: &str) -> usize {
        let bytes = string.as_bytes();
        bytes.iter().fold(0xcbf29ce484222325, |acc, byte| {
            (acc ^ (*byte as usize)).wrapping_mul(0x100000001b3)
        })
    }
}

impl Hashable for ObjectString {
    fn hash(&self) -> usize {
        self.hash
    }
}

impl<T> Hashable for Rc<T> where T: Hashable {
    fn hash(&self) -> usize {
        self.as_ref().hash()
    }
}

impl PartialEq for ObjectString {
    fn eq(&self, other: &Self) -> bool {
        if self.hash != other.hash || self.value.len() != other.value.len() {
            false
        } else {
            self.value == other.value
        }
    }
}

impl HashTable<Rc<ObjectString>, ()> {
    pub fn find_string_or_insert_new(&mut self, string: String) -> Rc<ObjectString> {
        let hash = ObjectString::hash_string(&string);
        let result = self
            .find_entry(hash, |key| key.as_ref().value == string)
            .map(|entry| Rc::clone(entry.entry_type.filled().unwrap()));
        match result {
            Some(string) => string,
            None => {
                let object = Rc::new(ObjectString::new(string, hash));
                let clone = Rc::clone(&object);
                self.insert(object, ());
                clone
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(ObjectString::hash_string("feedface"), 0x0a83c86fee952abc);
    }
}
