use super::vec::RawVec;
use super::value::Value;
use std::cmp::Eq;
use std::fmt::Debug;
use std::ptr;

const TABLE_MAX_LOAD: f32 = 0.75;

pub struct HashTable<Key: Hashable + Eq, Value> {
    length: usize,
    buffer: RawVec<Entry<Key, Value>>,
}

pub trait Hashable {
    fn hash(&self) -> usize;
}

struct Entry<Key: Eq, Value> {
    key: Option<Key>,
    value: Value,
}

impl<Key: Hashable + Eq + Debug, Value: Debug + Default> HashTable<Key, Value> {
    pub fn new() -> Self {
        Self {
            length: 0,
            buffer: RawVec::new(),
        }
    }
    pub fn insert(&mut self, key: Key, value: Value) {
        self.grow_if_needed();
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
        loop {
            unsafe {
                let entry = self.pointer().add(index).as_ref().unwrap();
                match &entry.key {
                    Some(entry_key) if key == entry_key => {
                        return Some(&entry.value);
                    },
                    _ => {
                        index = (index + 1) % self.buffer.capacity;
                    }
                }
                if initial_index == index {
                    return None;
                }
            };
        }
    }

    pub fn contains(&self, key: &Key) -> bool {
        self.find(key).is_some()
    }

    fn grow_if_needed(&mut self) {
        if self.length + 1 <= self.buffer.capacity * 75 / 100 {
            return;
        }
        self.buffer.grow();
        for index in 0..self.buffer.capacity + 1 {
            unsafe {
                ptr::write(self.pointer().add(index), Entry {
                    key: None,
                    value: Value::default()
                })
            }
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

    impl Hashable for String {
        fn hash(&self) -> usize {
            Value::hash_string(self)
        }
    }
}


