use super::value::Value;
use super::vec::Vec;
use super::op_code::OpCode;
use super::constant_pool::ConstantPool;
use std::mem;
use std::slice::Iter;

pub struct Chunk {
    pub codes: Vec<u8>,
    pub constants: ConstantPool,
    lines: Vec<LineStart>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            codes: Vec::new(),
            lines: Vec::new(),
            constants: ConstantPool::new()
        }
    }

    #[inline]
    pub fn byte_to_op_code(byte: u8) -> OpCode {
        unsafe {
            mem::transmute::<u8, OpCode>(byte)
        }
    }

    pub fn disassemble(self, name: String) {
        println!("== {} ==", name);
        let mut iter = self.codes.iter();
        let mut offset: usize = 0;
        while let Some(code) = iter.next() {
            let op_code = Chunk::byte_to_op_code(code.clone());
            offset += self.disassemble_instruction(&op_code, &mut iter, offset);
        }
    }

    pub fn push_code(&mut self, code: OpCode, line: usize) {
        println!("push code {}", &code);
        let code = unsafe {
            mem::transmute::<OpCode, u8>(code.clone())
        };
        self.push(code, line);
    }

    pub fn push_constant(&mut self, index: usize, line: usize) {
        println!("push constant at index {:?}", index);
        if index < 256 {
            self.push_code(OpCode::Constant, line);
            self.push(index as u8, line);
        } else {
            self.push_code(OpCode::ConstantLong, line);
            self.push((index & 0xff) as u8, line);
            self.push((index >> 8u8) as u8 & 0xff, line);
            self.push((index >> 16u8) as u8 & 0xff, line);
        }
    }

    pub fn add_constant(&mut self, constant: Value) -> usize {
        println!("add constant {:?}", &constant);
        self.constants.push(constant);
        self.constants.length() - 1
    }

    pub fn disassemble_instruction(&self, op_code: &OpCode, iter: &mut Iter<u8>, offset: usize) -> usize {
        let line = self.line(offset);
        match op_code {
            OpCode::Return | OpCode::Negate | OpCode::Add | OpCode::Subtract | OpCode::Multiply | OpCode::Divide => {
                println!("{} {} at {}", offset, op_code, line);
            }
            OpCode::False | OpCode::True | OpCode::Nil => {
                println!("{} {} at {}", offset, op_code, line);
            }
            OpCode::Constant => {
                let value = self.read_constant(iter);
                println!("{} {} {:?} at {}", offset, op_code, value, line);
            }
            OpCode::ConstantLong => {
                let value = self.read_constant_long(iter);
                println!("{} {} {:?} at {}", offset, op_code, value, line);
            }
        }
        offset + op_code.code_size()
    }

    #[inline]
    pub fn read_constant(&self, iterator: &mut Iter<u8>) -> Value {
        let index = iterator.next().unwrap().clone() as usize;
        self.constants.value(index)
    }

    #[inline]
    pub fn read_constant_long(&self, iterator: &mut Iter<u8>) -> Value {
        let index = iterator.next().unwrap().clone() as u32
            | u32::from(iterator.next().unwrap().clone()) << 8u8
            | u32::from(iterator.next().unwrap().clone()) << 16u8;
        self.constants.value(index as usize)
    }

    fn push(&mut self, byte: u8, line: usize) {
        self.codes.push(byte);
        match self.lines.last() {
            None => self.lines.push(LineStart { offset: 0, line }),
            Some(value) if value.line != line => {
                self.lines.push(LineStart { offset: self.codes.length - 1, line })
            },
            _ => {}
        }
    }

    pub fn line(&self, offset: usize) -> usize {
        let mut start = 0;
        let mut end = self.lines.length - 1;
        loop {
            let mid = (end + start) / 2;
            if offset < self.lines[mid].offset {
                end = mid - 1;
            } else if (mid == self.lines.length - 1) || (self.lines[mid + 1].offset > offset) {
                return self.lines[mid].line
            } else {
                start = mid + 1;
            }
        }
    }
}

struct LineStart {
    offset: usize,
    line: usize,
}
