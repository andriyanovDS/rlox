use std::mem;

const SIZE: usize = u8::MAX as usize + 1;
const NOT_INITIALIZED: Upvalue = Upvalue {
    index: 0,
    is_local: false
};

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Upvalue {
    pub index: u8,
    is_local: bool,
}

pub struct Upvalues {
    buffer: [Upvalue; SIZE],
    last_index: u8,
}

impl Upvalues {
    pub const fn new() -> Self {
        Self {
            buffer: [NOT_INITIALIZED; SIZE],
            last_index: 0,
        }
    }

    pub fn size(&self) -> u8 {
        self.last_index
    }

    pub fn push(&mut self, index: u8, is_local: bool) -> Option<u8> {
        let new_upvalue = Upvalue { index, is_local };
        let this = self as &Upvalues;
        for (index, upvalue) in this.into_iter().enumerate() {
            if upvalue == &new_upvalue {
                return Some(index as u8);
            }
        }
        if self.last_index == u8::MAX {
            None
        } else {
            let insertion_index = self.last_index;
            self.buffer[self.last_index as usize] = Upvalue { index, is_local };
            self.last_index += 1;
            Some(insertion_index)
        }
    }
}

pub struct UpvaluesRefIterator<'a> {
    upvalues: &'a Upvalues,
    index: u8,
    last_index: u8,
}

impl<'a> IntoIterator for &'a Upvalues {
    type Item = &'a Upvalue;
    type IntoIter = UpvaluesRefIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        UpvaluesRefIterator {
            upvalues: self,
            index: 0,
            last_index: self.last_index
        }
    }
}

pub struct UpvaluesIterator {
    upvalues: Upvalues,
    index: u8,
    last_index: u8,
}

impl<'a> Iterator for UpvaluesRefIterator<'a> {
    type Item = &'a Upvalue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.last_index {
            None
        } else {
            let upvalue = &self.upvalues.buffer[self.index as usize];
            self.index += 1;
            Some(upvalue)
        }
    }
}

impl Iterator for UpvaluesIterator {
    type Item = Upvalue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.last_index {
            None
        } else {
            let upvalue = mem::replace(&mut self.upvalues.buffer[self.index as usize], NOT_INITIALIZED);
            self.index += 1;
            Some(upvalue)
        }
    }
}

impl IntoIterator for Upvalues {
    type Item = Upvalue;
    type IntoIter = UpvaluesIterator;

    fn into_iter(self) -> Self::IntoIter {
        let last_index = self.last_index;
        UpvaluesIterator {
            upvalues: self,
            index: 0,
            last_index
        }
    }
}

impl PartialEq for Upvalues {
    fn eq(&self, other: &Self) -> bool {
        if self.last_index != other.last_index {
            false
        } else {
            self.into_iter()
                .zip(other.into_iter())
                .any(|(left, right)| left == right)
        }
    }
}

impl From<Upvalues> for Vec<Upvalue> {
    fn from(upvalues: Upvalues) -> Self {
        let mut vec: Vec<Upvalue> = Vec::with_capacity(upvalues.last_index as usize);
        for upvalue in upvalues.into_iter() {
            vec.push(upvalue)
        }
        vec
    }
}

#[cfg(test)]
mod tests {
    use crate::bytecode::upvalue::{Upvalue, Upvalues};

    #[test]
    fn test_that_upvalue_added_to_buffer() {
        let mut upvalues = Upvalues::new();
        upvalues.push(0, true);
        upvalues.push(1, true);
        upvalues.push(2, false);
        assert_eq!(Vec::from(upvalues), vec![
            Upvalue { index: 0, is_local: true },
            Upvalue { index: 1, is_local: true },
            Upvalue { index: 2, is_local: false },
        ])
    }

    #[test]
    fn test_that_upvalues_does_not_contain_duplicate_indices() {
        let mut upvalues = Upvalues::new();
        upvalues.push(0, false);
        upvalues.push(1, true);
        upvalues.push(1, true);
        upvalues.push(0, false);
        upvalues.push(0, true);

        assert_eq!(Vec::from(upvalues), vec![
            Upvalue { index: 0, is_local: false },
            Upvalue { index: 1, is_local: true },
            Upvalue { index: 0, is_local: true },
        ])
    }
}
