use super::value::Value;
use super::vec::Vec;
use super::op_code::OpCode;
use super::constant_pool::ConstantPool;
use std::mem;
use std::slice;

pub struct Chunk {
    codes: Vec<u8>,
    lines: Vec<LineStart>,
    constants: ConstantPool,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            codes: Vec::new(),
            lines: Vec::new(),
            constants: ConstantPool::new()
        }
    }

    pub fn disassemble(self, name: String) {
        println!("== {} ==", name);
        let mut iter = self.codes.iter();
        let mut index: usize = 0;
        while let Some(code) = iter.next() {
            index += 1;
            let line = self.line(index);
            let op_code = unsafe {
                mem::transmute::<u8, OpCode>(code.clone())
            };
            match op_code {
                OpCode::Return => println!("{} {} at {}", index, op_code, line),
                OpCode::Constant => {
                    let constant_index = iter.next().unwrap().clone() as usize;
                    index += 1;
                    println!("{} {} {:?} at {}", index, op_code, self.constants.value(constant_index), line);
                },
                OpCode::ConstantLong => {
                    let constant_index = iter.next().unwrap().clone() as u32
                        | u32::from(iter.next().unwrap().clone()) << 8u8
                        | u32::from(iter.next().unwrap().clone()) << 16u8;
                    index += 3;
                    println!("{} {} {:?} at {}", index, op_code, self.constants.value(constant_index as usize), line);
                }
            }
        }
    }

    pub fn push_code(&mut self, code: OpCode, line: usize) {
        let code = unsafe {
            mem::transmute::<OpCode, u8>(code.clone())
        };
        self.push(code, line);
    }

    pub fn push_constant(&mut self, index: usize, line: usize) {
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
        self.constants.push(constant);
        self.constants.length() - 1
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

    fn line(&self, offset: usize) -> usize {
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
