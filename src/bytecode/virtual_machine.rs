use super::value::Value;
use super::stack::Stack;
use super::op_code::OpCode;
use super::chunk::Chunk;
use std::ops::{Add, Sub, Mul, Div};

pub struct VirtualMachine {
    stack: Stack
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self { stack: Stack::new() }
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> InterpretResult {
        let mut offset: usize = 0;
        let mut iter = chunk.codes.iter();
        loop {
            if let Some(code) = iter.next() {
                let op_code = Chunk::byte_to_op_code(*code);
                offset += op_code.code_size();
                match op_code {
                    OpCode::Return => {
                        if let Some(value) = self.stack.pop() {
                            println!("{:?}", value);
                        }
                        break Ok(());
                    },
                    OpCode::Constant => {
                        let constant = chunk.read_constant(&mut iter);
                        self.stack.push(constant);
                    },
                    OpCode::ConstantLong => {
                        let constant = chunk.read_constant_long(&mut iter);
                        self.stack.push(constant);
                    },
                    OpCode::Negate => self.apply_negate_operation(offset, chunk)?,
                    OpCode::Add => self.apply_binary_operation(Add::add, offset, chunk)?,
                    OpCode::Subtract => self.apply_binary_operation(Sub::sub, offset, chunk)?,
                    OpCode::Multiply => self.apply_binary_operation(Mul::mul, offset, chunk)?,
                    OpCode::Divide => self.apply_binary_operation(Div::div, offset, chunk)?,
                    OpCode::True => self.stack.push(Value::Bool(true)),
                    OpCode::False => self.stack.push(Value::Bool(false)),
                    OpCode::Nil => self.stack.push(Value::Nil),
                    OpCode::Not => self.apply_not_operation(),
                }
            } else {
                break Ok(());
            }
        }
    }

    fn apply_binary_operation<F>(
        &mut self,
        operation: F,
        offset: usize,
        chunk: &Chunk,
    ) -> InterpretResult where F: FnOnce(f32, f32) -> f32 {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(Value::Number(right)), Some(Value::Number(left))) => {
                self.stack.push(Value::Number(operation(left, right)));
                Ok(())
            }
            _ => {
                self.runtime_error("Operands must be numbers.".to_string(), offset, chunk);
                Err(InterpretError::RuntimeError)
            }
        }
    }

    fn apply_negate_operation(&mut self, offset: usize, chunk: &Chunk) -> InterpretResult {
        let top_value = self.stack.peek_end(0).unwrap();
        if let Value::Number(number) = top_value {
            let new_number = -(*number);
            self.stack.modify_last(|_| Value::Number(new_number));
            Ok(())
        } else {
            self.runtime_error("Operand must be a number.".to_string(), offset, chunk);
            Err(InterpretError::RuntimeError)
        }
    }

    fn apply_not_operation(&mut self) {
        let top_value = self.stack.peek_end(0).unwrap();
        match top_value {
            Value::Bool(boolean) => {
                let new_value = !(*boolean);
                self.stack.modify_last(|_| Value::Bool(new_value));
            },
            Value::Nil => self.stack.modify_last(|_| Value::Bool(true)),
            _ => self.stack.modify_last(|_| Value::Bool(false)),
        }
    }

    fn runtime_error(&mut self, message: String, offset: usize, chunk: &Chunk) {
        eprintln!("{}", message);
        eprintln!("[line {}] in script.", chunk.line(offset));
    }
}

#[derive(Debug)]
pub enum InterpretError {
    CompileError,
    RuntimeError,
}

pub type InterpretResult = Result<(), InterpretError>;
