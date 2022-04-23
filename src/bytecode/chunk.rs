use super::value::Value;
use super::vec::Vec;
use super::op_code::OpCode;
use super::constant_pool::ConstantPool;
use std::mem;
use std::slice::Iter;

pub struct Chunk {
    pub codes: Vec<u8>,
    constants: ConstantPool,
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

    pub fn disassemble(&self, name: String) {
        println!("== {} ==", name);
        let mut iter = self.codes.iter();
        let mut offset: usize = 0;
        while let Some(code) = iter.next() {
            let op_code = Chunk::byte_to_op_code(code.clone());
            offset += self.disassemble_instruction(op_code, &mut iter, offset);
        }
    }

    pub fn push_code(&mut self, code: OpCode, line: usize) {
        println!("push code {}", &code);
        let code = unsafe {
            mem::transmute::<OpCode, u8>(code.clone())
        };
        self.push(code, line);
    }

    pub fn add_constant(&mut self, constant: Value, line: usize) {
        println!("add constant {:?}", &constant);
        let index = self.push_constant_to_pool(constant);
        self.push_constant(index, line);
    }

    #[inline]
    pub fn push_constant_to_pool(&mut self, constant: Value) -> usize {
        self.constants.push(constant);
        self.constants.length() - 1
    }

    pub fn disassemble_instruction(&self, op_code: OpCode, iter: &mut Iter<u8>, offset: usize) -> usize {
        let line = self.line(offset);
        let mut offset = offset;
        match op_code {
            OpCode::Return | OpCode::Negate | OpCode::Add
            | OpCode::Subtract | OpCode::Multiply | OpCode::Divide
            | OpCode::False | OpCode::True | OpCode::Nil
            | OpCode::Not | OpCode::Equal | OpCode::Greater
            | OpCode::Less | OpCode::Print | OpCode::Pop | OpCode::CloseUpvalue => {
                println!("{:04} {:4} {} at {}", offset, "", op_code, line);
            }
            OpCode::Constant | OpCode::DefineGlobal | OpCode::GetGlobal
            | OpCode::SetGlobal | OpCode::SetLocal | OpCode::GetLocal
            | OpCode::GetUpvalue | OpCode::SetUpvalue | OpCode::Class
            | OpCode::GetProperty | OpCode::SetProperty => {
                print!("{:04} ", offset);
                self.print_constant(iter, op_code, line);
            }
            OpCode::ConstantLong => {
                let value = self.read_constant_long(iter);
                println!("{:04} {} {:?} at {}", offset, op_code, value, line);
            }
            OpCode::JumpIfFalse | OpCode::Jump | OpCode::Loop => {
                let condition_offset = Chunk::read_condition_offset(iter);
                println!("{:04} {} {} at {}", offset, op_code, condition_offset, line)
            }
            OpCode::Call => {
                let argument_count = *(iter.next().unwrap());
                println!("{:04} {:4} {} {} at {}", offset, "", op_code, argument_count, line)
            }
            OpCode::Closure => {
                print!("{:04} ", offset);
                let value = self.print_constant(iter, op_code, line);
                offset += op_code.code_size();
                if let Value::Closure(closure) = value {
                    let count = closure.function.upvalue_count;
                    for _ in 0..count {
                        let is_local = *(iter.next().unwrap());
                        let index = *(iter.next().unwrap());
                        let locality = if is_local == 1 { "local" } else { "upvalue" };
                        println!("{:04}      |                     {} {}", offset, locality, index);
                        offset += 2;
                    }
                }
                return offset;
            }
        }
        offset + op_code.code_size()
    }

    #[inline]
    pub fn read_constant(&self, iterator: &mut Iter<u8>) -> &Value {
        let index = *iterator.next().unwrap() as usize;
        self.constants.value(index)
    }

    #[inline]
    pub fn read_constant_long(&self, iterator: &mut Iter<u8>) -> &Value {
        let index = *(iterator.next().unwrap()) as u32
            | u32::from(*(iterator.next().unwrap())) << 8u8
            | u32::from(*(iterator.next().unwrap())) << 16u8;
        self.constants.value(index as usize)
    }

    #[inline]
    pub fn read_condition_offset(iterator: &mut Iter<u8>) -> usize {
        usize::from(*(iterator.next().unwrap())) << 8u8
            | (*iterator.next().unwrap()) as usize

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

    pub fn push(&mut self, byte: u8, line: usize) {
        self.codes.push(byte);
        match self.lines.last() {
            None => self.lines.push(LineStart { offset: 0, line }),
            Some(value) if value.line != line => {
                self.lines.push(LineStart { offset: self.codes.length - 1, line })
            },
            _ => {}
        }
    }

    fn push_constant(&mut self, index: usize, line: usize) {
        println!("push constant at index {:?}", index);
        if index < 256 {
            self.push_code(OpCode::Constant, line);
            self.push(index as u8, line);
        } else {
            self.push_code(OpCode::ConstantLong, line);
            self.push((index & 0xff) as u8, line);
            self.push(((index >> 8u8) & 0xff) as u8, line);
            self.push(((index >> 8u8) & 0xff) as u8, line);
        }
    }

    #[inline]
    fn print_constant(&self, iterator: &mut Iter<u8>, op_code: OpCode, line: usize) -> &Value {
        let index = *iterator.next().unwrap() as usize;
        let constant = self.constants.value(index);
        println!("{:4} {} {:>16?} at {}", index, op_code, constant, line);
        constant
    }
}

struct LineStart {
    offset: usize,
    line: usize,
}
