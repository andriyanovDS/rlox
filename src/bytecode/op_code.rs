use std::fmt::{self, Display, Formatter};

#[derive(Copy, Clone)]
pub enum OpCode {
    Return,
    Constant,
    ConstantLong
}

impl Display for OpCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OpCode::Return => write!(f, "OP_RETURN"),
            OpCode::Constant => write!(f, "OP_CONSTANT"),
            OpCode::ConstantLong => write!(f, "OP_CONSTANT_LONG"),
        }
    }
}
