use std::alloc::{self, Layout};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::cmp::Eq;

pub struct RawTable<Key: Eq, Value> {
    pub pointer: NonNull<Entry<Key, Value>>,
    pub capacity: usize,
    _marker: PhantomData<Entry<Key, Value>>
}

pub trait Hashable {
    fn hash(&self) -> usize;
}

pub enum EntryType<Key> {
    Empty,
    Filled(Key),
    Deleted,
}

impl<Key> EntryType<Key> {
    fn filled(&self) -> Option<&Key> {
        match self {
            EntryType::Filled(key) => Some(key),
            _ => None,
        }
    }
}

pub struct Entry<Key: Eq, Value> {
    pub entry_type: EntryType<Key>,
    pub value: Value,
}

impl<Key: Eq, Value> Entry<Key, Value> {
    pub fn new(key: Key, value: Value) -> Self {
        Self {
            entry_type: EntryType::Filled(key),
            value
        }
    }
    pub fn deleted() -> Self where Value: Default {
        Self {
            entry_type: EntryType::Deleted,
            value: Value::default(),
        }
    }
}

impl<Key: Hashable + Eq, Value: Default> RawTable<Key, Value> {
    pub fn new() -> Self {
        RawTable {
            pointer: NonNull::dangling(),
            capacity: 0,
            _marker: PhantomData
        }
    }

    pub fn grow(&mut self) -> usize {
        let (new_capacity, new_layout) = if self.capacity == 0 {
            (1, Layout::array::<Entry<Key, Value>>(1).unwrap())
        } else {
            let new_capacity = self.capacity * 2;
            let new_layout = Layout::array::<Entry<Key, Value>>(new_capacity).unwrap();
            (new_capacity, new_layout)
        };
        assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");

        let (new_pointer, filled_entries_count) = unsafe {
            let pointer = alloc::alloc(new_layout) as *mut Entry<Key, Value>;
            RawTable::fill_new_table(&pointer, new_capacity);
            let filled_entries_count = if self.capacity > 0 {
                let count = self.move_items_to_new_table(&pointer, new_capacity);
                let layout = Layout::array::<Entry<Key, Value>>(self.capacity).unwrap();
                alloc::dealloc(self.pointer.as_ptr() as *mut u8, layout);
                count
            } else {
                0
            };
            (pointer, filled_entries_count)
        };
        self.pointer = match NonNull::new(new_pointer) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout)
        };
        self.capacity = new_capacity;
        filled_entries_count
    }

    unsafe fn fill_new_table(new_pointer: &*mut Entry<Key, Value>, new_capacity: usize) {
        for index in 0..new_capacity {
            new_pointer.add(index).write(Entry::default());
        }
    }

    unsafe fn move_items_to_new_table(
        &self,
        new_pointer: &*mut Entry<Key, Value>,
        new_capacity: usize
    ) -> usize {
        assert!(new_capacity > self.capacity);
        let old_pointer = self.pointer.as_ptr();
        (0..self.capacity)
            .into_iter()
            .map(|index| old_pointer.add(index).read())
            .fold(0, |acc, entry| {
                let index = entry.entry_type.filled().map(|v| v.hash() % new_capacity);
                if let Some(new_index) = index {
                    new_pointer.add(new_index).write(entry);
                    acc + 1
                } else {
                    acc
                }
            })
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

impl<Key> Default for EntryType<Key> {
    fn default() -> Self {
        Self::Empty
    }
}

impl<Key: Eq, Value: Default> Default for Entry<Key, Value> {
    fn default() -> Self {
        Self {
            entry_type: EntryType::Empty,
            value: Value::default(),
        }
    }
}