use op_code::OpCode;
use chunk::Chunk;
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

pub fn run_interpreter(script: String) {

    let mut scanner = Scanner::new(&script);
    loop {
        match scanner.scan_token() {
            Ok(token) => {
                println!("token {:?}, {:?}, {}", token.token_type, token.lexeme, token.line);
                if token.token_type == TokenType::Eof {
                    break;
                }
            },
            Err(error) => {
                println!("error: {}", error.message);
                break;
            }
        }
    }

    let mut chunk = Chunk::new();

    let index = chunk.add_constant(Value::Double(2.0));
    chunk.push_constant(index, 0);
    chunk.push_code(OpCode::Negate, 1);

    let index = chunk.add_constant(Value::Double(2.0));
    chunk.push_constant(index, 2);
    chunk.push_code(OpCode::Add, 2);

    chunk.push_code(OpCode::Return, 3);

    let mut virtual_machine = VirtualMachine::new();
    virtual_machine.interpret(&chunk);
}
