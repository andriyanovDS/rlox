use std::cell::RefCell;
use std::rc::Rc;
use crate::bytecode::compiler::Compiler;
use crate::bytecode::hash_table::HashTable;
use crate::bytecode::object_string::ObjectString;
use crate::bytecode::virtual_machine::VirtualMachine;

mod chunk;
mod op_code;
mod constant_pool;
mod value;
mod virtual_machine;
mod stack;
mod compiler;
mod scanner;
mod token;
mod parse_rule;
mod vec;
mod scope;
pub mod object_string;
pub mod hash_table;

pub fn run_interpreter(script: String) {
    let interned_strings = Rc::new(
        RefCell::new(HashTable::<Rc<ObjectString>, ()>::new())
    );

    let mut compiler = Compiler::new(&script, Rc::clone(&interned_strings));
    compiler.compile();

    let mut virtual_machine = VirtualMachine::new(Rc::clone(&interned_strings));
    if let Err(error) = virtual_machine.interpret(compiler.chunk()) {
        eprintln!("Interpret failed with error {:?}", error);
    }
}
