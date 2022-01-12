use super::value::Value;

const STACK_SIZE: usize = 256;
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
            let value = unsafe {
                std::mem::replace(&mut self.buffer[self.top_index], NOT_INITIALIZED)
            };
            Some(value)
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

    pub fn modify_last(&mut self, value: Value) {
        if self.top_index == 0 {
            return;
        }
        let index = self.top_index - 1;
        self.buffer[index] = value;
    }

    pub fn print_debug_info(&self) {
        println!("Index {}, Result {:?}", self.top_index, &self.buffer[..self.top_index])
    }
}
