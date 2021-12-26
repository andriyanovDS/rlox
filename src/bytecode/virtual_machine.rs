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
                }
            }
        }
        InterpretResult::Ok
    }
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}
