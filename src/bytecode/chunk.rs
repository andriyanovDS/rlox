use std::alloc::{alloc, Layout, realloc, dealloc, handle_alloc_error};
use std::marker::PhantomData;
use std::ops::{Deref};
use std::ptr::{self, NonNull};
use std::{slice, mem};
use std::fmt::{Display, Formatter};
use std::iter::{self, Iterator};
use super::op_code::OpCode;

struct Chunk {
    code: NonNull<OpCode>,
    length: usize,
    capacity: usize,
    _marker: PhantomData<OpCode>
}

pub struct IntoIter {
    buffer: NonNull<OpCode>,
    capacity: usize,
    start: *const OpCode,
    end: *const OpCode,
    _marker: PhantomData<OpCode>
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: NonNull::dangling(),
            length: 0,
            capacity: 0,
            _marker: PhantomData
        }
    }

    fn grow(&mut self) {
        let (new_capacity, new_layout) = if self.capacity == 0 {
            (1, Layout::array::<OpCode>(1).unwrap())
        } else {
            let new_capacity = self.capacity * 2;
            let new_layout = Layout::array::<OpCode>(new_capacity).unwrap();
            (new_capacity, new_layout)
        };
        assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");

        let new_pointer = if self.capacity == 0 {
            unsafe { alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<OpCode>(self.capacity).unwrap();
            let old_pointer = self.code.as_ptr() as *mut u8;
            unsafe { realloc(old_pointer, old_layout, new_layout.size()) }
        };
        self.code = match NonNull::new(new_pointer as *mut OpCode) {
            Some(p) => p,
            None => handle_alloc_error(new_layout)
        };
        self.capacity = new_capacity;
    }

    fn push(&mut self, code: OpCode) {
        if self.capacity == self.length {
            self.grow();
        }
        unsafe {
            ptr::write(self.code.as_ptr().add(self.length), code);
        }
        self.length += 1;
    }

    fn pop(&mut self) -> Option<OpCode> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            unsafe {
                Some(ptr::read(self.code.as_ptr().add(self.length)))
            }
        }
    }

    fn insert(&mut self, index: usize, code: OpCode) {
        assert!(index <= self.length, "index out of bounds");
        if self.capacity == self.length {
            self.grow();
        }
        unsafe {
            ptr::copy(
                self.code.as_ptr().add(index),
                self.code.as_ptr().add(index + 1),
                self.length - index
            );
            ptr::write(self.code.as_ptr().add(index), code);
        }
        self.length += 1;
    }

    fn remove(&mut self, index: usize) -> OpCode {
        assert!(index <= self.length, "index out of bounds");
        self.length -= 1;
        unsafe {
            let element = ptr::read(self.code.as_ptr().add(index));
            ptr::copy(
                self.code.as_ptr().add(index + 1),
                self.code.as_ptr().add(index),
                self.length - index
            );
            element
        }
    }

    fn disassemble_chunk(self, _name: String) {
        for code in self {
            println!("chunk {}", code);
        }
    }
}

impl iter::IntoIterator for Chunk {
    type Item = OpCode;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        let buffer = self.code;
        let capacity = self.capacity;
        let length = self.length;

        mem::forget(self);

        unsafe {
            IntoIter {
                buffer,
                capacity,
                start: buffer.as_ptr(),
                end: if capacity == 0 {
                    buffer.as_ptr()
                } else {
                    buffer.as_ptr().add(length)
                },
                _marker: PhantomData
            }
        }
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        if self.capacity != 0 {
            while let Some(_) = self.pop() {}
            let layout = Layout::array::<OpCode>(self.capacity).unwrap();
            unsafe {
                dealloc(self.code.as_ptr() as *mut u8, layout);
            }
        }
    }
}

impl Deref for Chunk {
    type Target = [OpCode];

    fn deref(&self) -> &[OpCode] {
        unsafe {
            slice::from_raw_parts(self.code.as_ptr(), self.length)
        }
    }
}

impl Iterator for IntoIter {
    type Item = OpCode;

    fn next(&mut self) -> Option<OpCode> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let next = ptr::read(self.start);
                self.start = self.start.offset(1);
                Some(next)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = (self.end as usize - self.start as usize) / mem::size_of::<OpCode>();
        (length, Some(length))
    }
}

impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<OpCode> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                self.end = self.end.offset(-1);
                Some(ptr::read(self.end))
            }
        }
    }
}
