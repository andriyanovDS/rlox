mod raw_table;

use raw_table::{RawTable, Hashable, Entry, EntryType};
use std::cmp::Eq;
use std::collections::HashMap;

const TABLE_MAX_LOAD: f32 = 0.75;

pub struct HashTable<Key: Hashable + Eq, Value> {
    length: usize,
    buffer: RawTable<Key, Value>,
}

impl<Key: Hashable + Eq, Value: Default> HashTable<Key, Value> {
    pub fn new() -> Self {
        Self {
            length: 0,
            buffer: RawTable::new(),
        }
    }

    pub fn insert(&mut self, key: Key, value: Value) {
        self.grow_if_needed();
        let mut index = key.hash() % self.buffer.capacity;
        unsafe {
            let mut pointer = self.pointer().add(index);
            let mut tombstone_index: Option<usize> = None;
            loop {
                let entry = pointer.as_ref().unwrap();
                match &entry.entry_type {
                    EntryType::Filled(entry_key) if &key == entry_key => { break; },
                    EntryType::Empty => {
                        if let Some(tombstone_index) = tombstone_index {
                            pointer = self.pointer().add(tombstone_index);
                        } else {
                            self.length += 1;
                        }
                        break;
                    }
                    EntryType::Deleted if tombstone_index.is_none() => {
                        tombstone_index = Some(index);
                        index += 1;
                        pointer = self.pointer().add(self.make_index(index));
                    }
                    _ => {
                        index += 1;
                        pointer = self.pointer().add(self.make_index(index));
                    }
                }
            }
            pointer.write(Entry::new(key, value));
        }
    }

    pub fn contains(&self, key: &Key) -> bool {
        self.find(key).is_some()
    }

    pub fn find(&self, key: &Key) -> Option<&Value> {
        if self.length == 0 {
            return None;
        }
        let mut index = self.make_index(key.hash());
        let initial_index = index;
        loop {
            unsafe {
                let entry = self.pointer().add(index).as_ref().unwrap();
                match &entry.entry_type {
                    EntryType::Empty => { break None; }
                    EntryType::Filled(entry_key) if entry_key == key => {
                        break Some(&entry.value);
                    }
                    _ => {
                        index = self.make_index(index + 1);
                    }
                }
                if index == initial_index {
                    break None;
                }
            };
        }
    }

    pub fn remove(&mut self, key: &Key) -> Option<Value> {
        if self.length == 0 {
            return None;
        }
        let hash = key.hash();
        let mut index = self.make_index(hash);
        loop {
            unsafe  {
                let pointer = self.pointer().add(index);
                let entry = pointer.as_ref().unwrap();
                match &entry.entry_type {
                    EntryType::Empty => { return None; }
                    EntryType::Filled(entry_key) if entry_key == key => {
                        let value = pointer.read().value;
                        pointer.write(Entry::deleted());
                        return Some(value);
                    }
                    _ => {
                        index = self.make_index(index + 1);
                    }
                }
            }
        }
    }

    pub fn clone_all(&self, destination: &mut Self) where Key: Clone, Value: Clone {
        assert_eq!(destination.length, 0);
        unsafe {
            for index in 0..self.buffer.capacity {
                let entry = self.pointer().add(index).as_ref().unwrap();
                if let EntryType::Filled(ref key) = entry.entry_type {
                    destination.insert(key.clone(), entry.value.clone());
                }
            }
        }
    }

    #[inline]
    fn make_index(&self, from_index: usize) -> usize {
        from_index % self.buffer.capacity
    }

    fn grow_if_needed(&mut self) {
        if self.length + 1 > self.buffer.capacity * 75 / 100 {
            self.length = self.buffer.grow();
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
        let first_key = "something".to_string();
        let second_key = "other".to_string();
        hash_map.insert(first_key.clone(), Value::Bool(true));
        hash_map.insert(second_key.clone(), Value::Number(1f32));
        hash_map.insert("other_key_3".to_string(), Value::Number(1f32));
        assert_eq!(hash_map.contains(&first_key), true);
        assert_eq!(hash_map.contains(&second_key), true);
        assert_eq!(hash_map.contains(&"not_inserted_key".to_string()), false);
    }

    #[test]
    fn remove() {
        let mut hash_map = HashTable::<String, Value>::new();
        let first_key = "something".to_string();
        let first_value = Value::Bool(true);
        let second_key = "other".to_string();
        let second_value = Value::Number(1f32);

        hash_map.insert(first_key.clone(), first_value.clone());
        hash_map.insert(second_key.clone(), second_value.clone());

        let first_removed = hash_map.remove(&first_key);
        assert_eq!(hash_map.contains(&second_key), true);

        let second_removed = hash_map.remove(&second_key);

        assert_eq!(hash_map.contains(&first_key), false);
        assert_eq!(hash_map.contains(&second_key), false);
        assert_eq!(first_removed.unwrap(), first_value);
        assert_eq!(second_removed.unwrap(), second_value);
    }

    #[test]
    fn clone_all() {
        let mut hash_map = HashTable::<String, Value>::new();
        let mut clone_map = HashTable::<String, Value>::new();
        let first_key = "something".to_string();
        let second_key = "other".to_string();

        hash_map.insert(first_key.clone(), Value::Bool(true));
        hash_map.insert(second_key.clone(), Value::Number(1f32));
        hash_map.clone_all(&mut clone_map);

        assert_eq!(clone_map.contains(&first_key), true);
        assert_eq!(clone_map.contains(&second_key), true);
        assert_eq!(hash_map.contains(&first_key), true);
        assert_eq!(hash_map.contains(&second_key), true);
    }

    impl Hashable for String {
        fn hash(&self) -> usize {
            Value::hash_string(self)
        }
    }
}


