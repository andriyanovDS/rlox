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

pub const FRAMES_SIZE: usize = 64;
pub struct VirtualMachine {
    stack: Stack,
    interned_strings: Rc<RefCell<HashTable<Rc<ObjectString>, ()>>>,
    globals: HashTable<Rc<ObjectString>, Value>,
    frame_count: usize,
}

impl VirtualMachine {
    pub fn new(interned_strings: Rc<RefCell<HashTable<Rc<ObjectString>, ()>>>, ) -> Self {
         Self {
             stack: Stack::new(),
             interned_strings,
             globals: HashTable::new(),
             frame_count: 0,
        }
    }

    pub fn interpret(&mut self, chunk: &Chunk) {
        if let Err(error) = self.handle_chunk(chunk, 0) {
            eprintln!("[line {}] in script", chunk.line(error.0))
        }
    }

    fn handle_chunk(&mut self, chunk: &Chunk, slots_start: usize) -> InterpretResult {
        self.frame_count += 1;
        assert!(self.frame_count < FRAMES_SIZE);

        let mut iter = chunk.codes.iter();
        let mut offset: usize = 0;
        loop {
            if let Some(code) = iter.next() {
                let op_code = Chunk::byte_to_op_code(*code);
                let prev_offset = offset;
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
                    OpCode::Negate => self.apply_negate_operation(prev_offset)?,
                    OpCode::Add => self.apply_add_operation(prev_offset)?,
                    OpCode::Subtract => self.apply_binary_operation(Sub::sub, prev_offset)?,
                    OpCode::Multiply => self.apply_binary_operation(Mul::mul, prev_offset)?,
                    OpCode::Divide => self.apply_binary_operation(Div::div, prev_offset)?,
                    OpCode::True => self.stack.push(Value::Bool(true)),
                    OpCode::False => self.stack.push(Value::Bool(false)),
                    OpCode::Nil => self.stack.push(Value::Nil),
                    OpCode::Not => self.apply_not_operation(),
                    OpCode::Equal => self.apply_equal_operation(),
                    OpCode::Greater => self.apply_compare_operation(|a, b| a > b, prev_offset)?,
                    OpCode::Less => self.apply_compare_operation(|a, b| a < b, prev_offset)?,
                    OpCode::Print => println!("{:?}", self.stack.pop().unwrap()),
                    OpCode::Pop => { self.stack.pop(); },
                    OpCode::DefineGlobal => self.define_global_variable(chunk, &mut iter),
                    OpCode::GetGlobal => self.get_global_variable(chunk, &mut iter, prev_offset)?,
                    OpCode::SetGlobal => self.set_global_variable(chunk, &mut iter, prev_offset)?,
                    OpCode::GetLocal => self.get_local_variable(&mut iter, slots_start),
                    OpCode::SetLocal => self.set_local_variable(&mut iter, slots_start),
                    OpCode::JumpIfFalse => self.handle_jump_if_false(&mut iter, &mut offset),
                    OpCode::Jump => {
                        let jump_offset = Chunk::read_condition_offset(&mut iter);
                        if jump_offset > 0 {
                            iter.nth(jump_offset - 1);
                            offset += jump_offset;
                        }
                    }
                    OpCode::Loop => {
                        let jump_offset = Chunk::read_condition_offset(&mut iter);
                        iter = chunk.codes.iter();
                        iter.nth(offset - jump_offset - 1);
                        offset -= jump_offset;
                    }
                    OpCode::Call => self.handle_function_call(&mut iter, prev_offset)?,
                }
            } else {
                break Ok(());
            }
        }
    }

    fn apply_binary_operation<F>(
        &mut self,
        operation: F,
        offset: usize
    ) -> InterpretResult where F: FnOnce(f32, f32) -> f32 {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(Value::Number(right)), Some(Value::Number(left))) => {
                self.stack.push(Value::Number(operation(left, right)));
                Ok(())
            }
            _ => {
                Err(VirtualMachine::runtime_error("Operands must be numbers.".to_string(), offset))
            }
        }
    }

    fn apply_add_operation(&mut self, offset: usize) -> InterpretResult {
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
                Err(VirtualMachine::runtime_error("Operands must be two numbers or two strings.".to_string(), offset))
            }
        }
    }

    fn apply_compare_operation<F>(
        &mut self,
        operation: F,
        offset: usize,
    ) -> InterpretResult where F: FnOnce(f32, f32) -> bool {
        match (self.stack.pop().unwrap(), self.stack.pop().unwrap()) {
            (Value::Number(right), Value::Number(left)) => {
                self.stack.push(Value::Bool(operation(left, right)));
                Ok(())
            }
            _ => {
                Err(VirtualMachine::runtime_error("Operands must be numbers.".to_string(), offset))
            }
        }
    }

    fn apply_negate_operation(&mut self, offset: usize) -> InterpretResult {
        let top_value = self.stack.peek_end(0).unwrap();
        if let Value::Number(number) = top_value {
            let new_number = -(*number);
            self.stack.modify_last(Value::Number(new_number));
            Ok(())
        } else {
            Err(VirtualMachine::runtime_error("Operand must be a number.".to_string(), offset))
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

    #[inline]
    fn get_global_variable(
        &mut self,
        chunk: &Chunk,
        iter: &mut Iter<u8>,
        offset: usize
    ) -> InterpretResult {
        if let Value::String(object) = chunk.read_constant(iter) {
            match self.globals.find(object) {
                Some(variable) => {
                    self.stack.push(variable.clone());
                    Ok(())
                }
                None => {
                    let variable = &object.as_ref().value;
                    Err(VirtualMachine::runtime_error(format!("Undefined variable {:?}", variable), offset))
                }
            }
        } else {
            panic!("Unexpected value type in global variable");
        }
    }

    #[inline]
    fn set_global_variable(
        &mut self,
        chunk: &Chunk,
        iter: &mut Iter<u8>,
        offset: usize
    ) -> InterpretResult {
        if let Value::String(object) = chunk.read_constant(iter) {
            if !self.globals.contains(object) {
                let variable = &object.as_ref().value;
                Err(VirtualMachine::runtime_error(format!("Undefined variable {:?}", variable), offset))
            } else {
                let value = self.stack.peek_end(0).unwrap();
                self.globals.insert(Rc::clone(object), value.clone());
                Ok(())
            }
        } else {
            panic!("Unexpected value type in global variable");
        }
    }

    #[inline]
    fn get_local_variable(&mut self, iter: &mut Iter<u8>, slots_start: usize) {
        let index = *(iter.next().unwrap()) as usize;
        self.stack.push(self.stack.copy_value(slots_start + index));
    }

    #[inline]
    fn set_local_variable(&mut self, iter: &mut Iter<u8>, slots_start: usize) {
        let index = *(iter.next().unwrap()) as usize;
        let value = self.stack.peek_end(0).unwrap().clone();
        self.stack.modify_at_index(slots_start + index, value);
    }

    #[inline]
    fn handle_jump_if_false(&mut self, iter: &mut Iter<u8>, offset: &mut usize) {
        let jump_offset = Chunk::read_condition_offset(iter);
        let top_value = self.stack.peek_end(0).unwrap();
        match top_value {
            Value::Bool(false) | Value::Nil => {
                iter.nth(jump_offset - 1);
                *offset += jump_offset;
            },
            _ => {},
        }
    }

    #[inline]
    fn handle_function_call(
        &mut self,
        iter: &mut Iter<u8>,
        offset: usize
    ) -> InterpretResult {
        let arguments_count = *(iter.next().unwrap());
        let arguments_count_usize = arguments_count as usize;
        let function = self.stack.peek_end(arguments_count_usize);

        match function.unwrap() {
            Value::Function(func) if func.arity != arguments_count => {
                let err_message = format!(
                    "{:?} function expects {} arguments but got {}.",
                    func.name,
                    func.arity,
                    arguments_count
                );
                Err(VirtualMachine::runtime_error(err_message, offset))
            }
            Value::Function(func) => {
                if self.frame_count + 1 == FRAMES_SIZE {
                    return Err(VirtualMachine::runtime_error("Stack overflow.".to_string(), offset));
                }
                let cloned_function = Rc::clone(func);
                let slots_start = self.stack.top_index() - arguments_count_usize;
                let chunk = &cloned_function.as_ref().chunk;
                let result =  self.handle_chunk(chunk, slots_start);
                self.frame_count -= 1;
                result.map_err(|_| {
                    eprintln!("[line {}] in {:?}()", chunk.line(offset), cloned_function.name);
                    InterpretError(offset)
                })
            }
            _ => {
                Err(VirtualMachine::runtime_error("Can only call functions and classes.".to_string(), offset))
            }
        }
    }

    fn runtime_error(message: String, offset: usize) -> InterpretError {
        eprintln!("{}", message);
        InterpretError(offset)
    }
}

#[derive(Debug)]
pub struct InterpretError(usize);
pub type InterpretResult = Result<(), InterpretError>;
