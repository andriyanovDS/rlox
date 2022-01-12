use std::mem;
use std::ptr;

pub struct RawValIter<Element> {
    start: *const Element,
    end: *const Element,
}

impl<Element> RawValIter<Element> {
    pub unsafe fn new(slice: &[Element]) -> Self {
        return Self {
            start: slice.as_ptr(),
            end: if slice.len() == 0 {
                slice.as_ptr()
            } else {
                slice.as_ptr().add(slice.len())
            }
        }
    }
}

impl<Element> Iterator for RawValIter<Element> {
    type Item = Element;

    fn next(&mut self) -> Option<Element> {
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
        let length = (self.end as usize - self.start as usize) / mem::size_of::<Element>();
        (length, Some(length))
    }
}

impl<Element> DoubleEndedIterator for RawValIter<Element> {
    fn next_back(&mut self) -> Option<Element> {
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
