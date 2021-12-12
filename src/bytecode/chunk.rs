use super::vec::Vec;
use super::op_code::OpCode;

pub struct Chunk {
    codes: Vec<OpCode>
}

impl Chunk {
    pub fn new() -> Self {
        Self { codes: Vec::new() }
    }

    pub fn disassemble(self, name: String) {
        println!("== {} ==", name);
        for (index, code) in self.codes.iter().enumerate() {
            println!("{} {}", index, code);
        }
    }

    pub fn push(&mut self, code: OpCode) {
        self.codes.push(code)
    }

    pub fn pop(&mut self) -> Option<OpCode> {
        self.codes.pop()
    }
}
