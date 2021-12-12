use std::fmt::{self, Display, Formatter, write};

pub enum OpCode {
    Return,
    SomeOther,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OpCode::Return => write!(f, "OP_RETURN"),
            OpCode::SomeOther => write!(f, "Some_Other"),
        }
    }
}
