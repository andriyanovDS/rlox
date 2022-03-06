use std::cell::RefCell;
use std::rc::Rc;
use crate::bytecode::compiler::{Compiler, CompilerContext};
use crate::bytecode::hash_table::HashTable;
use crate::bytecode::object_string::ObjectString;
use crate::bytecode::scanner::Scanner;
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
mod object_function;

pub fn run_interpreter(script: String) {
    let interned_strings = Rc::new(
        RefCell::new(HashTable::<Rc<ObjectString>, ()>::new())
    );

    let scanner = Rc::new(RefCell::new(Scanner::new(&script)));
    let parse_rules = Compiler::make_parse_rules();
    let compiler_context = CompilerContext::new(
        Rc::clone(&scanner),
        &script,
        &parse_rules,
        Rc::clone(&interned_strings)
    );
    let mut compiler = Compiler::new(compiler_context);
    if let Some(chunk) = compiler.compile() {
        let mut virtual_machine = VirtualMachine::new(Rc::clone(&interned_strings));
        virtual_machine.interpret(chunk);
    }
}
