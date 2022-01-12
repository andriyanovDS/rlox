use std::alloc::{self, Layout};
use std::marker::PhantomData;
use std::ptr::NonNull;

pub struct RawVec<Element> {
    pub pointer: NonNull<Element>,
    pub capacity: usize,
    _marker: PhantomData<Element>
}

impl<Element> RawVec<Element> {
    pub fn new() -> Self {
        RawVec {
            pointer: NonNull::dangling(),
            capacity: 0,
            _marker: PhantomData
        }
    }

    pub fn grow(&mut self) {
        let (new_capacity, new_layout) = if self.capacity == 0 {
            (1, Layout::array::<Element>(1).unwrap())
        } else {
            let new_capacity = self.capacity * 2;
            let new_layout = Layout::array::<Element>(new_capacity).unwrap();
            (new_capacity, new_layout)
        };
        assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");

        let new_pointer = if self.capacity == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<Element>(self.capacity).unwrap();
            let old_pointer = self.pointer.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_pointer, old_layout, new_layout.size()) }
        };
        self.pointer = match NonNull::new(new_pointer as *mut Element) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout)
        };
        self.capacity = new_capacity;
    }
}

impl<Element> Drop for RawVec<Element> {
    fn drop(&mut self) {
        if self.capacity != 0 {
            let layout = Layout::array::<Element>(self.capacity).unwrap();
            unsafe {
                alloc::dealloc(self.pointer.as_ptr() as *mut u8, layout);
            }
        }
    }
}
