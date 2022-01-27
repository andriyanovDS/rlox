mod raw_table;

use super::vec::RawVec;
use super::value::Value;
use raw_table::{RawTable, Hashable, Entry};
use std::cmp::Eq;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ptr;

const TABLE_MAX_LOAD: f32 = 0.75;

pub struct HashTable<Key: Hashable + Eq, Value> {
    length: usize,
    buffer: RawTable<Key, Value>,
}

impl<Key: Hashable + Eq + Debug, Value: Debug + Default> HashTable<Key, Value> {
    pub fn new() -> Self {
        Self {
            length: 0,
            buffer: RawTable::new(),
        }
    }
    pub fn insert(&mut self, key: Key, value: Value) {
        self.grow_if_needed();
        self.length += 1;
        let index = key.hash() % self.buffer.capacity;
        unsafe {
            let mut pointer = self.pointer().add(index);
            loop {
                let entry = ptr::read(pointer);
                match entry.key {
                    Some(entry_key) if key == entry_key => {
                        break;
                    },
                    None => {
                        break;
                    }
                    _ => {
                        pointer = self.pointer().add((index + 1) % self.buffer.capacity)
                    }
                }
            }
            ptr::write(pointer, Entry { key: Some(key), value });
        }
    }

    pub fn find(&self, key: &Key) -> Option<&Value> {
        let mut index = key.hash() % self.buffer.capacity;
        let initial_index = index;
        println!("index {}", initial_index);
        loop {
            println!("next index {}", index);
            unsafe {
                // println!("pointer: {:?}", self.pointer().add(index));
                // let entry = self.pointer().add(index);
                // println!("entry {:?}", entry.key);
                return None;
            };
        }
    }

    pub fn contains(&self, key: &Key) -> bool {
        self.find(key).is_some()
    }

    fn grow_if_needed(&mut self) {
        println!("grow if needed {} - {}", self.buffer.capacity, self.buffer.capacity * 75 / 100);
        if self.length + 1 > self.buffer.capacity * 75 / 100 {
            self.buffer.grow();
        }
    }

    fn pointer(&self) -> *mut Entry<Key, Value> {
        self.buffer.pointer.as_ptr()
    }
}

#[cfg(test)]
mod tests {
    use crate::bytecode::value::Value;
    use super::*;

    #[test]
    fn insertion() {
        let mut hash_map = HashTable::<String, Value>::new();
        hash_map.insert("some_key".to_string(), Value::Bool(true));
        assert_eq!(hash_map.contains(&"some_key".to_string()), true)
    }

    #[test]
    fn find() {
        let mut hash_map = HashTable::<String, Value>::new();
        hash_map.insert("some_key".to_string(), Value::Bool(true));
        let value = hash_map.find(&"some_key".to_string()).map(|v| v.clone());
        assert_eq!(value, Some(Value::Bool(true)));
    }

    #[test]
    fn resize() {
        let mut hash_map = HashTable::<String, Value>::new();
        let first_key = "some_key".to_string();
        let second_key = "other_key".to_string();
        hash_map.insert(first_key.clone(), Value::Bool(true));
        hash_map.insert(second_key.clone(), Value::Number(1f32));
        hash_map.insert("other_key_3".to_string(), Value::Number(1f32));
        // assert_eq!(hash_map.contains(&first_key), true);
        // assert_eq!(hash_map.contains(&"other_key".to_string()), true)
    }

    impl Hashable for String {
        fn hash(&self) -> usize {
            Value::hash_string(self)
        }
    }
}


