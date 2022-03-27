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
        match self {
            OpCode::Return => write!(f, "OP_RETURN"),
            OpCode::Constant => write!(f, "OP_CONSTANT"),
            OpCode::ConstantLong => write!(f, "OP_CONSTANT_LONG"),
            OpCode::Negate => write!(f, "OP_NEGATE"),
            OpCode::Add => write!(f, "OP_ADD"),
            OpCode::Subtract => write!(f, "OP_SUBTRACT"),
            OpCode::Multiply => write!(f, "OP_MULTIPLY"),
            OpCode::Divide => write!(f, "OP_DIVIDE"),
            OpCode::True => write!(f, "OP_TRUE"),
            OpCode::False => write!(f, "OP_FALSE"),
            OpCode::Nil => write!(f, "OP_NIL"),
            OpCode::Not => write!(f, "OP_NOT"),
            OpCode::Greater => write!(f, "OP_GREATER"),
            OpCode::Equal => write!(f, "OP_EQUAL"),
            OpCode::Less => write!(f, "OP_LESS"),
            OpCode::Print => write!(f, "OP_PRINT"),
            OpCode::Pop => write!(f, "OP_POP"),
            OpCode::DefineGlobal => write!(f, "OP_DEFINE_GLOBAL"),
            OpCode::GetGlobal => write!(f, "OP_GET_GLOBAL"),
            OpCode::SetGlobal => write!(f, "OP_SET_GLOBAL"),
            OpCode::GetLocal => write!(f, "OP_GET_LOCAL"),
            OpCode::SetLocal => write!(f, "OP_SET_LOCAL"),
            OpCode::GetUpvalue => write!(f, "OP_GET_UPVALUE"),
            OpCode::SetUpvalue => write!(f, "OP_SET_UPVALUE"),
            OpCode::JumpIfFalse => write!(f, "OP_JUMP_IF_FALSE"),
            OpCode::Jump => write!(f, "OP_JUMP"),
            OpCode::Loop => write!(f, "OP_LOOP"),
            OpCode::Call => write!(f, "OP_CALL"),
            OpCode::Closure => write!(f, "OP_CLOSURE"),
        }
    }
}
