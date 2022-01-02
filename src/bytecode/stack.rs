use super::value::Value;

const STACK_SIZE: usize = 256;
pub struct Stack {
    buffer: [Value; STACK_SIZE],
    top_index: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            buffer: [Value::Double(0f32); STACK_SIZE],
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
            Some(self.buffer[self.top_index])
        }
    }

    pub fn modify_last<F>(&mut self, modifier: F) where F: FnOnce(Value) -> Value {
        if self.top_index == 0 {
            return;
        }
        let index = self.top_index - 1;
        self.buffer[index] = modifier(self.buffer[index]);
    }
}
