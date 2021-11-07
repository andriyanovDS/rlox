use crate::interpreter::Interpreter;
use crate::parser::Parser;
use error::Error;
use io::{BufRead, Error as IOError, Write};
use resolver::Resolver;
use scanner::Scanner;
use std::cell::RefCell;
use std::rc::Rc;
use std::{fs, io, result::Result};

mod callable;
mod clock;
mod environment;
mod error;
mod expression;
mod interpreter;
mod lox_function;
mod native_function;
mod object;
mod parser;
mod resolver;
mod scanner;
mod statement;
mod token;
mod token_type;
mod lox_class;

pub fn run_prompt() -> Result<(), IOError> {
    print!("> ");
    io::stdout().flush().unwrap();

    for read_result in io::stdin().lock().lines() {
        let line = read_result?;
        run_interpreter(line);

        print!("> ");
        io::stdout().flush().unwrap();
    }
    Ok(())
}

pub fn run_file(path: String) {
    let content = fs::read_to_string(path).expect("File not found");
    run_interpreter(content);
}

fn run_interpreter(script: String) {
    let mut scanner = Scanner::new(script.as_str());
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(&tokens);
    let statements = parser.parse();

    if statements.is_empty() {
        return;
    }

    let interpreter = Rc::new(RefCell::new(Interpreter::new()));
    let mut resolver = Resolver::new(interpreter.clone());
    match resolver.resolve_statements(&statements) {
        Err(error) => eprintln!("{}", error.description()),
        _ => {
            interpreter.as_ref().borrow_mut().interpret(&statements);
        }
    }
}
