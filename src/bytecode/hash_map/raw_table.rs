use std::alloc::{self, Layout};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::cmp::Eq;
use std::fmt::Debug;
use std::mem;

pub struct RawTable<Key: Eq, Value> {
    pub pointer: NonNull<Entry<Key, Value>>,
    pub capacity: usize,
    _marker: PhantomData<Entry<Key, Value>>
}

pub trait Hashable {
    fn hash(&self) -> usize;
}

pub struct Entry<Key: Eq, Value> {
    pub key: Option<Key>,
    pub value: Value,
}

impl<Key: Hashable + Eq, Value: Default> RawTable<Key, Value> {
    pub fn new() -> Self {
        RawTable {
            pointer: NonNull::dangling(),
            capacity: 0,
            _marker: PhantomData
        }
    }

    pub fn grow(&mut self) {
        let (new_capacity, new_layout) = if self.capacity == 0 {
            (1, Layout::array::<Entry<Key, Value>>(1).unwrap())
        } else {
            let new_capacity = self.capacity * 2;
            let new_layout = Layout::array::<Entry<Key, Value>>(new_capacity).unwrap();
            (new_capacity, new_layout)
        };
        assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");

        let new_pointer = unsafe {
            let pointer = alloc::alloc(new_layout) as *mut Entry<Key, Value>;
            RawTable::fill_new_table(&pointer, new_capacity);
            if self.capacity > 0 {
                self.move_items_to_new_table(&pointer, new_capacity);
                let layout = Layout::array::<Entry<Key, Value>>(self.capacity).unwrap();
                alloc::dealloc(self.pointer.as_ptr() as *mut u8, layout);
            }
            pointer
        };
        self.pointer = match NonNull::new(new_pointer) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout)
        };
        self.capacity = new_capacity;
    }

    unsafe fn fill_new_table(new_pointer: &*mut Entry<Key, Value>, new_capacity: usize) {
        for index in 0..new_capacity {
            new_pointer.add(index).write(Entry {
                key: None,
                value: Value::default()
            });
        }
    }

    unsafe fn move_items_to_new_table(&self, new_pointer: &*mut Entry<Key, Value>, new_capacity: usize) {
        assert!(new_capacity > self.capacity);
        let old_pointer = self.pointer.as_ptr();
        let mut index = 0usize;
        while index < self.capacity {
            let entry = old_pointer.add(index).read();
            index += 1;
            match &entry.key {
                None => {
                    continue;
                },
                Some(key) => {
                    let new_index = key.hash() % new_capacity;
                    new_pointer.add(new_index).write(entry);
                }
            }
        }
    }
}

impl<Key: Eq, Value> Drop for RawTable<Key, Value> {
    fn drop(&mut self) {
        if self.capacity != 0 {
            let layout = Layout::array::<Entry<Key, Value>>(self.capacity).unwrap();
            unsafe {
                alloc::dealloc(self.pointer.as_ptr() as *mut u8, layout);
            }
        }
    }
}