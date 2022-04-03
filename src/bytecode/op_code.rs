use std::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, PartialEq)]
pub enum OpCode {
    Return,
    Constant,
    ConstantLong,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    True,
    False,
    Nil,
    Not,
    Greater,
    Equal,
    Less,
    Print,
    Pop,
    DefineGlobal,
    GetGlobal,
    SetGlobal,
    GetLocal,
    SetLocal,
    GetUpvalue,
    SetUpvalue,
    JumpIfFalse,
    Jump,
    Loop,
    Call,
    Closure,
    CloseUpvalue,
}

impl OpCode {
    pub fn code_size(&self) -> usize {
        match self {
            OpCode::Constant | OpCode::DefineGlobal | OpCode::GetGlobal | OpCode::Closure
            | OpCode::SetGlobal | OpCode::SetLocal | OpCode::GetLocal | OpCode::Call
            | OpCode::GetUpvalue | OpCode::SetUpvalue => 2,
            OpCode::JumpIfFalse | OpCode::Loop | OpCode::Jump => 3,
            OpCode::ConstantLong => 4,
            _ => 1
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let representation = match self {
            OpCode::Return => "OP_RETURN",
            OpCode::Constant => "OP_CONSTANT",
            OpCode::ConstantLong => "OP_CONSTANT_LONG",
            OpCode::Negate => "OP_NEGATE",
            OpCode::Add => "OP_ADD",
            OpCode::Subtract => "OP_SUBTRACT",
            OpCode::Multiply => "OP_MULTIPLY",
            OpCode::Divide => "OP_DIVIDE",
            OpCode::True => "OP_TRUE",
            OpCode::False => "OP_FALSE",
            OpCode::Nil => "OP_NIL",
            OpCode::Not => "OP_NOT",
            OpCode::Greater => "OP_GREATER",
            OpCode::Equal => "OP_EQUAL",
            OpCode::Less => "OP_LESS",
            OpCode::Print => "OP_PRINT",
            OpCode::Pop => "OP_POP",
            OpCode::DefineGlobal => "OP_DEFINE_GLOBAL",
            OpCode::GetGlobal => "OP_GET_GLOBAL",
            OpCode::SetGlobal => "OP_SET_GLOBAL",
            OpCode::GetLocal => "OP_GET_LOCAL",
            OpCode::SetLocal => "OP_SET_LOCAL",
            OpCode::GetUpvalue => "OP_GET_UPVALUE",
            OpCode::SetUpvalue => "OP_SET_UPVALUE",
            OpCode::JumpIfFalse => "OP_JUMP_IF_FALSE",
            OpCode::Jump => "OP_JUMP",
            OpCode::Loop => "OP_LOOP",
            OpCode::Call => "OP_CALL",
            OpCode::Closure => "OP_CLOSURE",
            OpCode::CloseUpvalue => "OP_CLOSE_UPVALUE"
        };
        write!(f, "{:<16}", representation)
    }
}
