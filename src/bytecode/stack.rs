use super::virtual_machine::FRAMES_SIZE;
use super::value::Value;

const U8_MAX: usize = u8::MAX as usize + 1;
const STACK_SIZE: usize = FRAMES_SIZE * U8_MAX;
const NOT_INITIALIZED: Value = Value::Nil;
pub struct Stack {
    buffer: [Value; STACK_SIZE],
    top_index: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            buffer: [NOT_INITIALIZED; STACK_SIZE],
            top_index: 0,
        }
    }

    pub fn top_index(&self) -> usize { self.top_index }

    pub fn push(&mut self, value: Value) {
        if self.top_index == STACK_SIZE - 1 {
            panic!("Stack overflow")
        } else {
            self.buffer[self.top_index] = value;
            self.top_index += 1;
        }
    }

    pub fn pop(&mut self) -> Option<Value> {
        if self.top_index == 0 {
            None
        } else {
            self.top_index -= 1;
            Some(std::mem::replace(&mut self.buffer[self.top_index], NOT_INITIALIZED))
        }
    }

    pub fn reset(&mut self) {
        self.top_index = 0;
    }

    pub fn peek_end(&self, distance: usize) -> Option<&Value> {
        if self.top_index == 0 || distance > self.top_index {
            None
        } else {
            Some(&self.buffer[self.top_index - 1 - distance])
        }
    }

    #[inline]
    pub fn modify_last(&mut self, value: Value) {
        self.modify_at_index(self.top_index - 1, value);
    }

    #[inline]
    pub fn modify_at_index(&mut self, index: usize, value: Value) {
        if self.top_index == 0 {
            return;
        }
        self.buffer[index] = value;
    }

    #[inline]
    pub fn copy_value(&self, index: usize) -> Value {
        assert!(index <= self.top_index);
        self.buffer[index].clone()
    }

    #[inline]
    pub fn value_at(&mut self, index: usize) -> &mut Value {
        assert!(index <= self.top_index);
        &mut self.buffer[index]
    }

    pub fn print_debug_info(&self) {
        println!("Index {}, Result {:?}", self.top_index, &self.buffer[..self.top_index])
    }
}
