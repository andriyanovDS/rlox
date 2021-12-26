use std::mem;
use crate::bytecode::value::Value;
use super::stack::Stack;
use super::op_code::OpCode;
use super::chunk::Chunk;

pub struct VirtualMachine {
    stack: Stack
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self { stack: Stack::new() }
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> InterpretResult {
        let mut iter = chunk.codes.iter();
        while let Some(code) = iter.next() {
            let op_code = Chunk::byte_to_op_code(code.clone());
            match op_code {
                OpCode::Return => {
                    if let Some(value) = self.stack.pop() {
                        println!("{:?}", value);
                    }
                    return InterpretResult::Ok;
                },
                OpCode::Constant => {
                    let constant = chunk.read_constant(&mut iter);
                    self.stack.push(constant);
                },
                OpCode::ConstantLong => {
                    let constant = chunk.read_constant_long(&mut iter);
                    self.stack.push(constant);
                },
                OpCode::Negate => {
                    match self.stack.pop() {
                        Some(Value::Double(value)) => {
                            self.stack.push(Value::Double(-value));
                        },
                        _ => {},
                    };
                },
                OpCode::Add => self.apply_binary_operation(|left, right| left + right),
                OpCode::Subtract => self.apply_binary_operation(|left, right| left - right),
                OpCode::Multiply => self.apply_binary_operation(|left, right| left * right),
                OpCode::Divide => self.apply_binary_operation(|left, right| left / right),
            }
        }
        InterpretResult::Ok
    }

    fn apply_binary_operation<F>(&mut self, operation: F) where F: FnOnce(f32, f32) -> f32 {
        let value = match (self.stack.pop(), self.stack.pop()) {
            (Some(Value::Double(right)), Some(Value::Double(left))) => Value::Double(operation(left, right)),
            _ => panic!("Unexpected values")
        };
        self.stack.push(value);
    }
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}
