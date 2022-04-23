use std::cell::{RefCell};
use std::cmp::Ordering;
use super::stack::Stack;
use super::op_code::OpCode;
use super::chunk::Chunk;
use super::hash_table::HashTable;
use super::value::{Value, object_string::ObjectString};
use std::time::{SystemTime, UNIX_EPOCH};
use std::ops::{Sub, Mul, Div};
use std::rc::Rc;
use std::slice::Iter;
use std::collections::BinaryHeap;
use super::value::object_instance::ObjectInstance;
use super::value::object_class::ObjectClass;
use super::value::object_closure::ObjectClosure;
use super::value::object_native_function::ObjectNativeFunction;
use super::value::object_upvalue::ObjectUpvalue;

pub const FRAMES_SIZE: usize = 64;
pub struct VirtualMachine {
    stack: Stack,
    interned_strings: Rc<RefCell<HashTable<Rc<ObjectString>, ()>>>,
    globals: HashTable<Rc<ObjectString>, Value>,
    frame_count: usize,
    open_upvalues: BinaryHeap<Rc<RefCell<ObjectUpvalue>>>,
}

impl VirtualMachine {
    pub fn new(interned_strings: Rc<RefCell<HashTable<Rc<ObjectString>, ()>>>,) -> Self {
        let mut globals = HashTable::new();
        VirtualMachine::add_native_functions(&mut globals, &interned_strings);
        Self {
            stack: Stack::new(),
            interned_strings,
            globals,
            frame_count: 0,
            open_upvalues: BinaryHeap::new(),
        }
    }

    pub fn interpret(&mut self, chunk: &Chunk) {
        let upvalue = Vec::new();
        if let Err(error) = self.handle_chunk(chunk, 0, &Vec::new(), &upvalue) {
            eprintln!("[line {}] in script", chunk.line(error.0))
        }
    }

    fn add_native_functions(
        globals: &mut HashTable<Rc<ObjectString>, Value>,
        interned_strings: &Rc<RefCell<HashTable<Rc<ObjectString>, ()>>>
    ) {
        let string = ObjectString::from_string("clock".to_string());
        let rc_string = Rc::new(string);
        let mut mut_interned_strings = interned_strings.as_ref().borrow_mut();
        mut_interned_strings.insert(Rc::clone(&rc_string),());
        globals.insert(Rc::clone(&rc_string), Value::NativeFunction(ObjectNativeFunction {
            function: Box::new(|| {
                let system_time = SystemTime::now();
                let milliseconds = system_time.duration_since(UNIX_EPOCH).unwrap().as_millis();
                Value::Number(milliseconds as f32)
            })
        }));
    }

    fn handle_chunk(
        &mut self,
        chunk: &Chunk,
        slots_start: usize,
        upvalues: &[Rc<RefCell<ObjectUpvalue>>],
        enclosing_upvalues: &[Rc<RefCell<ObjectUpvalue>>]
    ) -> InterpretResult {
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
                    OpCode::Return => {
                        self.close_upvalue(self.stack.top_index());
                        break Ok(());
                    },
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
                    OpCode::GetUpvalue => self.get_upvalue(&mut iter, &upvalues),
                    OpCode::SetUpvalue => self.set_upvalue(&mut iter, upvalues),
                    OpCode::GetProperty => self.get_property(chunk, &mut iter, prev_offset)?,
                    OpCode::SetProperty => self.set_property(&mut iter, slots_start),
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
                    OpCode::Call => self.handle_call(&mut iter, prev_offset, &upvalues)?,
                    OpCode::Closure => {
                        self.read_closure(chunk, &mut offset, &mut iter, slots_start, enclosing_upvalues)
                    },
                    OpCode::CloseUpvalue => {
                        self.close_upvalue(self.stack.top_index());
                        self.stack.pop();
                    },
                    OpCode::Class => self.read_class(chunk, &mut iter),
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
    fn get_upvalue(&mut self, iter: &mut Iter<u8>, upvalues: &[Rc<RefCell<ObjectUpvalue>>]) {
        let slot = *(iter.next().unwrap()) as usize;
        self.stack.push(upvalues[slot].as_ref().borrow().value().clone())
    }

    #[inline]
    fn set_upvalue(&mut self, iter: &mut Iter<u8>, upvalues: &[Rc<RefCell<ObjectUpvalue>>]) {
        let slot = *(iter.next().unwrap()) as usize;
        let value = self.stack.peek_end(0).unwrap();
        upvalues[slot].as_ref().borrow_mut().set_value(value.clone())
    }

    #[inline]
    fn get_property(&mut self, chunk: &Chunk, iter: &mut Iter<u8>, offset: usize) -> InterpretResult {
        let top_value = self.stack.peek_end(0).unwrap();
        let constant = chunk.read_constant(iter);
        match (top_value, constant) {
            (Value::Instance(instance), Value::String(object)) => {
                match instance.as_ref().property(object) {
                    Some(value) => {
                        let value = value.clone();
                        self.stack.pop();
                        self.stack.push(value.clone());
                        Ok(())
                    }
                    None => {
                        Err(VirtualMachine::runtime_error(format!("Undefined property {:?}", object), offset))
                    }
                }
            }
            (Value::Instance(_), _) => panic!("Unexpected value type instead of instance property name"),
            _ => Err(VirtualMachine::runtime_error("Only instances have properties".to_string(), offset))
        }
    }

    #[inline]
    fn set_property(&mut self, iter: &mut Iter<u8>, slots_start: usize) {

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
    fn handle_call(
        &mut self,
        iter: &mut Iter<u8>,
        offset: usize,
        upvalues: &[Rc<RefCell<ObjectUpvalue>>],
    ) -> InterpretResult {
        let arguments_count = *(iter.next().unwrap());
        let arguments_count_usize = arguments_count as usize;
        let callee = self.stack.peek_end(arguments_count_usize);

        match callee.unwrap() {
            Value::Closure(closure) if closure.function.arity != arguments_count => {
                let func = &closure.function;
                Err(VirtualMachine::runtime_error(
                    format!("{:?} function expects {} arguments but got {}.", func.name, func.arity, arguments_count),
                    offset
                ))
            }
            Value::Closure(closure) => {
                let closure = Rc::clone(closure);
                self.call_closure(&closure, arguments_count_usize, offset, &closure.upvalues, upvalues)
            },
            Value::NativeFunction(object) => {
                let result: Value = (object.function)();
                self.stack.push(result);
                Ok(())
            }
            Value::Class(class) => {
                let instance = ObjectInstance::new(class.clone());
                self.stack.push(Value::Instance(Rc::new(instance)));
                Ok(())
            }
            _ => {
                Err(VirtualMachine::runtime_error("Can only call functions and classes.".to_string(), offset))
            }
        }
    }

    fn read_closure(
        &mut self,
        chunk: &Chunk,
        offset: &mut usize,
        iter: &mut Iter<u8>,
        slots_start: usize,
        enclosing_upvalues: &[Rc<RefCell<ObjectUpvalue>>]
    ) {
        let constant = chunk.read_constant(iter);
        if let Value::Function(function) = constant {
            let mut upvalues = Vec::new();

            for _ in 0..function.upvalue_count {
                let is_local = if *(iter.next().unwrap()) == 1u8 { true } else { false };
                let index = *(iter.next().unwrap());
                if is_local {
                    let upvalue = self.capture_upvalue(slots_start + index as usize);
                    upvalues.push(upvalue);
                } else {
                    upvalues.push(enclosing_upvalues[index as usize].clone());
                }
                *offset += 2;
            }

            let closure = ObjectClosure {
                function: Rc::clone(function),
                upvalues,
            };
            self.stack.push(Value::Closure(Rc::new(closure)));

        } else {
            panic!("Unexpected value found instead of ObjectFunction")
        }
    }

    fn capture_upvalue(&mut self, index: usize) -> Rc<RefCell<ObjectUpvalue>> {
        let value = self.stack.value_at(index);
        let object_upvalue = ObjectUpvalue::new(value);

        let existing_upvalue = self.open_upvalues
            .iter()
            .find(|v| v.as_ref().borrow().eq(&object_upvalue));

        match existing_upvalue {
            Some(upvalue) => upvalue.clone(),
            None => {
                let captured_upvalue = Rc::new(RefCell::new(object_upvalue));
                self.open_upvalues.push(captured_upvalue.clone());
                captured_upvalue
            }
        }
    }

    fn close_upvalue(&mut self, top_index: usize) {
        let value = self.stack.value_at(top_index);
        let object_upvalue = ObjectUpvalue::new(value);
        loop {
            let ordering = self.open_upvalues.peek().map(|v| object_upvalue.cmp(&v.as_ref().borrow()));
            match ordering {
                Some(Ordering::Equal) => {},
                Some(Ordering::Greater) => {},
                _ => {
                    break;
                }
            }
            let upvalue = self.open_upvalues.pop().unwrap();
            let mut upvalue = upvalue.as_ref().borrow_mut();
            let value = upvalue.value().clone();
            upvalue.close_value(value.clone())
        }
    }

    #[inline]
    fn call_closure(
        &mut self,
        closure: &ObjectClosure,
        arguments_count: usize,
        offset: usize,
        upvalues: &[Rc<RefCell<ObjectUpvalue>>],
        enclosing_upvalues: &[Rc<RefCell<ObjectUpvalue>>]
    ) -> InterpretResult {
        if self.frame_count + 1 == FRAMES_SIZE {
            return Err(VirtualMachine::runtime_error("Stack overflow.".to_string(), offset));
        }
        let cloned_function = Rc::clone(&closure.function);
        let slots_start = self.stack.top_index() - arguments_count;
        let chunk = &cloned_function.as_ref().chunk;

        let result = self.handle_chunk(chunk, slots_start, upvalues, enclosing_upvalues);
        let return_value = self.stack.pop().unwrap();
        while self.stack.top_index() + 1 > slots_start {
            self.stack.pop();
        }
        self.stack.push(return_value);
        self.frame_count -= 1;
        result.map_err(|_| {
            eprintln!("[line {}] in {:?}()", chunk.line(offset), cloned_function.name);
            InterpretError(offset)
        })
    }

    #[inline]
    fn read_class(&mut self, chunk: &Chunk, iter: &mut Iter<u8>) {
        if let Value::String(object) = chunk.read_constant(iter) {
            let class_object = ObjectClass::new(object.clone());
            self.stack.push(Value::Class(Rc::new(class_object)));
        } else {
            panic!("Unexpected value type instead of class declaration");
        }
    }

    #[inline]
    fn runtime_error(message: String, offset: usize) -> InterpretError {
        eprintln!("{}", message);
        InterpretError(offset)
    }
}

#[derive(Debug)]
pub struct InterpretError(usize);
pub type InterpretResult = Result<(), InterpretError>;
