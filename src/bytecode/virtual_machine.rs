use std::cell::RefCell;
use super::value::Value;
use super::stack::Stack;
use super::op_code::OpCode;
use super::chunk::Chunk;
use super::hash_table::HashTable;
use super::object_string::ObjectString;
use std::ops::{Sub, Mul, Div};
use std::rc::Rc;
use std::slice::Iter;

pub struct VirtualMachine {
    stack: Stack,
    interned_strings: Rc<RefCell<HashTable<Rc<ObjectString>, ()>>>,
    globals: HashTable<Rc<ObjectString>, Value>,
}

impl VirtualMachine {
    pub fn new(interned_strings: Rc<RefCell<HashTable<Rc<ObjectString>, ()>>>) -> Self {
         Self {
            stack: Stack::new(),
             interned_strings,
             globals: HashTable::new(),
        }
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> InterpretResult {
        let mut offset: usize = 0;
        let mut iter = chunk.codes.iter();
        loop {
            if let Some(code) = iter.next() {
                let op_code = Chunk::byte_to_op_code(*code);
                offset += op_code.code_size();
                match op_code {
                    OpCode::Return => { break Ok(()); },
                    OpCode::Constant => {
                        let constant = chunk.read_constant(&mut iter);
                        self.stack.push(constant.clone());
                    },
                    OpCode::ConstantLong => {
                        let constant = chunk.read_constant_long(&mut iter);
                        self.stack.push(constant.clone());
                    },
                    OpCode::Negate => self.apply_negate_operation(offset, chunk)?,
                    OpCode::Add => self.apply_add_operation(offset, chunk)?,
                    OpCode::Subtract => self.apply_binary_operation(Sub::sub, offset, chunk)?,
                    OpCode::Multiply => self.apply_binary_operation(Mul::mul, offset, chunk)?,
                    OpCode::Divide => self.apply_binary_operation(Div::div, offset, chunk)?,
                    OpCode::True => self.stack.push(Value::Bool(true)),
                    OpCode::False => self.stack.push(Value::Bool(false)),
                    OpCode::Nil => self.stack.push(Value::Nil),
                    OpCode::Not => self.apply_not_operation(),
                    OpCode::Equal => self.apply_equal_operation(),
                    OpCode::Greater => self.apply_compare_operation(|a, b| a > b, offset, chunk)?,
                    OpCode::Less => self.apply_compare_operation(|a, b| a < b, offset, chunk)?,
                    OpCode::Print => println!("{:?}", self.stack.pop().unwrap()),
                    OpCode::Pop => { self.stack.pop(); },
                    OpCode::DefineGlobal => self.define_global_variable(chunk, &mut iter)
                }
            } else {
                self.stack.print_debug_info();
                break Ok(());
            }
        }
    }

    fn apply_binary_operation<F>(
        &mut self,
        operation: F,
        offset: usize,
        chunk: &Chunk,
    ) -> InterpretResult where F: FnOnce(f32, f32) -> f32 {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(Value::Number(right)), Some(Value::Number(left))) => {
                self.stack.push(Value::Number(operation(left, right)));
                Ok(())
            }
            _ => {
                self.runtime_error("Operands must be numbers.".to_string(), offset, chunk);
                Err(InterpretError::RuntimeError)
            }
        }
    }

    fn apply_add_operation(&mut self, offset: usize, chunk: &Chunk) -> InterpretResult {
        match (self.stack.pop().unwrap(), self.stack.pop().unwrap()) {
            (Value::Number(right), Value::Number(left)) => {
                self.stack.push(Value::Number(left + right));
                Ok(())
            }
            (Value::String(right), Value::String(left)) => {
                let string = left.as_ref().value.clone() + right.as_ref().value.as_str();
                let mut strings = self.interned_strings.as_ref().borrow_mut();
                let object = strings.find_string_or_insert_new(string);
                self.stack.push(Value::String(object));
                Ok(())
            }
            _ => {
                self.runtime_error("Operands must be two numbers or two strings.".to_string(), offset, chunk);
                Err(InterpretError::RuntimeError)
            }
        }
    }

    fn apply_compare_operation<F>(
        &mut self,
        operation: F,
        offset: usize,
        chunk: &Chunk,
    ) -> InterpretResult where F: FnOnce(f32, f32) -> bool {
        match (self.stack.pop().unwrap(), self.stack.pop().unwrap()) {
            (Value::Number(right), Value::Number(left)) => {
                self.stack.push(Value::Bool(operation(left, right)));
                Ok(())
            }
            _ => {
                self.runtime_error("Operands must be numbers.".to_string(), offset, chunk);
                Err(InterpretError::RuntimeError)
            }
        }
    }

    fn apply_negate_operation(&mut self, offset: usize, chunk: &Chunk) -> InterpretResult {
        let top_value = self.stack.peek_end(0).unwrap();
        if let Value::Number(number) = top_value {
            let new_number = -(*number);
            self.stack.modify_last(Value::Number(new_number));
            Ok(())
        } else {
            self.runtime_error("Operand must be a number.".to_string(), offset, chunk);
            Err(InterpretError::RuntimeError)
        }
    }

    fn apply_not_operation(&mut self) {
        let top_value = self.stack.peek_end(0).unwrap();
        match top_value {
            Value::Bool(boolean) => {
                let new_value = !(*boolean);
                self.stack.modify_last(Value::Bool(new_value));
            },
            Value::Nil => self.stack.modify_last(Value::Bool(true)),
            _ => self.stack.modify_last(Value::Bool(false)),
        }
    }

    fn apply_equal_operation(&mut self) {
        let left = self.stack.pop().unwrap();
        let right = self.stack.pop().unwrap();
        self.stack.push(Value::Bool(left == right));
    }

    #[inline]
    fn define_global_variable(&mut self, chunk: &Chunk, iter: &mut Iter<u8>) {
        let value = chunk.read_constant(iter);
        if let Value::String(object) = value {
            let value = self.stack.pop().unwrap();
            self.globals.insert(Rc::clone(object), value);
        } else {
            panic!("Unexpected value type in global variable");
        }
    }

    fn runtime_error(&mut self, message: String, offset: usize, chunk: &Chunk) {
        eprintln!("{}", message);
        eprintln!("[line {}] in script.", chunk.line(offset));
    }
}

#[derive(Debug)]
pub enum InterpretError {
    CompileError,
    RuntimeError,
}

pub type InterpretResult = Result<(), InterpretError>;
