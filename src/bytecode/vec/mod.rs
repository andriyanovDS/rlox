mod raw_val_iter;
mod raw_vec;

use std::marker::PhantomData;
use std::ops::{Deref};
use std::ptr;
use std::{slice, mem};
use std::iter;
use super::vec::raw_val_iter::RawValIter;
pub use super::vec::raw_vec::RawVec;

pub struct Vec<Element> {
    buffer: RawVec<Element>,
    pub length: usize,
}

impl<Element> Vec<Element> {
    pub fn new() -> Self {
        Self {
            buffer: RawVec::new(),
            length: 0,
        }
    }

    pub fn push(&mut self, element: Element) {
        if self.capacity() == self.length {
            self.buffer.grow();
        }
        unsafe {
            ptr::write(self.pointer().add(self.length), element);
        }
        self.length += 1;
    }

    pub fn pop(&mut self) -> Option<Element> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            unsafe {
                Some(ptr::read(self.pointer().add(self.length)))
            }
        }
    }

    pub fn insert(&mut self, index: usize, element: Element) {
        assert!(index <= self.length, "index out of bounds");
        if self.capacity() == self.length {
            self.buffer.grow();
        }
        unsafe {
            ptr::copy(
                self.pointer().add(index),
                self.pointer().add(index + 1),
                self.length - index
            );
            ptr::write(self.pointer().add(index), element);
        }
        self.length += 1;
    }

    pub fn remove(&mut self, index: usize) -> Element {
        assert!(index <= self.length, "index out of bounds");
        self.length -= 1;
        unsafe {
            let element = ptr::read(self.pointer().add(index));
            ptr::copy(
                self.pointer().add(index + 1),
                self.pointer().add(index),
                self.length - index
            );
            element
        }
    }

    fn capacity(&self) -> usize {
        self.buffer.capacity
    }

    fn pointer(&self) -> *mut Element {
        self.buffer.pointer.as_ptr()
    }
}

impl<Element> Drop for Vec<Element> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
    }
}

impl<Element> Deref for Vec<Element> {
    type Target = [Element];

    fn deref(&self) -> &[Element] {
        unsafe {
            slice::from_raw_parts(self.pointer(), self.length)
        }
    }
}

pub struct IntoIter<Element> {
    _buffer: RawVec<Element>,
    iter: RawValIter<Element>,
}

impl<Element> iter::IntoIterator for Vec<Element> {
    type Item = Element;
    type IntoIter = IntoIter<Element>;

    fn into_iter(self) -> IntoIter<Element> {
        unsafe {
            let iter = RawValIter::new(&self);
            let raw_vec = ptr::read(&self.buffer);
            mem::forget(self);
            IntoIter { _buffer: raw_vec, iter }
        }
    }
}

impl<Element> Iterator for IntoIter<Element> {
    type Item = Element;

    fn next(&mut self) -> Option<Element> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<Element> DoubleEndedIterator for IntoIter<Element> {
    fn next_back(&mut self) -> Option<Element> {
        self.iter.next_back()
    }
}

impl<Element> Drop for IntoIter<Element> {
    fn drop(&mut self) {
        for _ in &mut *self {}
    }
}

pub struct Drain<'a, Element> {
    vec: PhantomData<&'a mut Vec<Element>>,
    iter: RawValIter<Element>
}

impl<'a, Element> Iterator for Drain<'a, Element> {
    type Item = Element;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, Element> DoubleEndedIterator for Drain<'a, Element> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<'a, Element> Drop for Drain<'a, Element> {
    fn drop(&mut self) {
        for _ in &mut *self {}
    }
}

impl<Element> Vec<Element> {
    fn drain(&mut self) -> Drain<Element> {
        unsafe {
            let iter = RawValIter::new(&self);
            self.length = 0;
            Drain { vec: PhantomData, iter }
        }
    }
}
