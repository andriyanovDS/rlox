use op_code::OpCode;
use chunk::Chunk;
use crate::bytecode::compiler::Compiler;
use crate::bytecode::scanner::Scanner;
use crate::bytecode::token::TokenType;
use crate::bytecode::value::Value;
use crate::bytecode::virtual_machine::VirtualMachine;

mod chunk;
mod op_code;
mod raw_vec;
mod raw_val_iter;
mod vec;
mod constant_pool;
mod value;
mod virtual_machine;
mod stack;
mod compiler;
mod scanner;
mod token;
mod parse_rule;

pub fn run_interpreter(script: String) {
    let mut compiler = Compiler::new(&script);
    compiler.compile();
    let mut virtual_machine = VirtualMachine::new();
    if let Err(error) = virtual_machine.interpret(compiler.chunk()) {
        eprintln!("Interpret failed with error {:?}", error);
    }
}
