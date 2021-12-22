use op_code::OpCode;
use chunk::Chunk;
use crate::bytecode::value::Value;

mod chunk;
mod op_code;
mod raw_vec;
mod raw_val_iter;
mod vec;
mod constant_pool;
mod value;

pub fn run_interpreter(_script: String) {
    let mut chunk = Chunk::new();
    chunk.push_code(OpCode::Return, 0);
    let index = chunk.add_constant(Value::Double(1.3));
    chunk.push_code(OpCode::Constant, 1);
    chunk.push_constant(index, 1);

    for index in 0..259 {
        let index = chunk.add_constant(Value::Double(index as f32));
        chunk.push_constant(index, 2);
    }

    chunk.disassemble("test chunk".to_string());
}
