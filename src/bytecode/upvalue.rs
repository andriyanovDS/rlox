use super::compiler::{CompilationResult, CompileError};

const SIZE: usize = u8::MAX as usize + 1;
const NOT_INITIALIZED: Upvalue = Upvalue {
    index: 0,
    is_local: false
};

pub struct Upvalue {
    pub index: u8,
    is_local: bool,
}

pub struct Upvalues {
    upvalues: [Upvalue; SIZE],
    last_index: u8,
}

impl Upvalues {
    pub const fn new() -> Self {
        Self {
            upvalues: [NOT_INITIALIZED; SIZE],
            last_index: 0,
        }
    }

    pub fn size(&self) -> u8 {
        self.last_index
    }

    pub fn push(&mut self, index: u8, is_local: bool) {
        assert!(self.last_index < u8::MAX);
        self.upvalues[self.last_index as usize] = Upvalue { index, is_local };
        self.last_index += 1;
    }
}
