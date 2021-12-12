use op_code::OpCode;
use chunk::Chunk;

mod chunk;
mod op_code;
mod raw_vec;
mod raw_val_iter;
mod vec;

pub fn run_interpreter(_script: String) {
    let mut chunk = Chunk::new();
    chunk.push(OpCode::Return);
    chunk.push(OpCode::SomeOther);
    chunk.disassemble("test chunk".to_string());
}
