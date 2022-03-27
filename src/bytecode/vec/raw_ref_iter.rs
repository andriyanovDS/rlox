use std::marker::PhantomData;
use std::mem;

pub struct RawRefIter<'a, Element> {
    start: *const Element,
    end: *const Element,
    _phantom_data: PhantomData<&'a Element>
}

impl<'a, Element> RawRefIter<'a, Element> {
    pub unsafe fn new(slice: &[Element]) -> Self {
        return Self {
            start: slice.as_ptr(),
            end: if slice.len() == 0 {
                slice.as_ptr()
            } else {
                slice.as_ptr().add(slice.len())
            },
            _phantom_data: PhantomData
        }
    }
}

impl<'a, Element> Iterator for RawRefIter<'a, Element> {
    type Item = &'a Element;

    fn next(&mut self) -> Option<&'a Element> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let next= self.start.as_ref();
                self.start = self.start.offset(1);
                next
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = (self.end as usize - self.start as usize) / mem::size_of::<Element>();
        (length, Some(length))
    }
}
