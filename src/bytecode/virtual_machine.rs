use super::value::Value;
use super::stack::Stack;
use super::op_code::OpCode;
use super::chunk::Chunk;

pub struct VirtualMachine {
    stack: Stack
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self { stack: Stack::new() }
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> InterpretResult {
        let mut offset: usize = 0;
        let mut iter = chunk.codes.iter();
        while let Some(code) = iter.next() {
            let op_code = Chunk::byte_to_op_code(code.clone());
            offset += op_code.code_size();
            match op_code {
                OpCode::Return => {
                    if let Some(value) = self.stack.pop() {
                        println!("{:?}", value);
                    }
                    return InterpretResult::Ok;
                },
                OpCode::Constant => {
                    let constant = chunk.read_constant(&mut iter);
                    self.stack.push(constant);
                },
                OpCode::ConstantLong => {
                    let constant = chunk.read_constant_long(&mut iter);
                    self.stack.push(constant);
                },
                OpCode::Negate => {
                    let top_value = self.stack.peek_end(0).unwrap();
                    if let Value::Number(number) = top_value {
                        let new_number = -(*number);
                        self.stack.modify_last(|_| Value::Number(new_number));
                    } else {
                        self.runtime_error("Operand must be a number.".to_string(), offset, chunk);
                        return InterpretResult::RuntimeError;
                    }
                },
                OpCode::Add => self.apply_binary_operation(|left, right| left + right),
                OpCode::Subtract => self.apply_binary_operation(|left, right| left - right),
                OpCode::Multiply => self.apply_binary_operation(|left, right| left * right),
                OpCode::Divide => self.apply_binary_operation(|left, right| left / right),
            }
        }
        InterpretResult::Ok
    }

    fn apply_binary_operation<F>(&mut self, operation: F) where F: FnOnce(f32, f32) -> f32 {
        let value = match (self.stack.pop(), self.stack.pop()) {
            (Some(Value::Number(right)), Some(Value::Number(left))) => Value::Number(operation(left, right)),
            _ => panic!("Unexpected values")
        };
        self.stack.push(value);
    }

    fn runtime_error(&mut self, message: String, offset: usize, chunk: &Chunk) {
        eprintln!("{}", message);
        eprintln!("[line {}] in script.", chunk.line(offset));
    }
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}
