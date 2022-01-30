use super::compiler::{Compiler, CompilationResult};
use std::convert::TryFrom;
use std::cmp::{Ordering, PartialOrd, PartialEq};

#[derive(Copy, Clone, Debug)]
pub enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl PartialOrd for Precedence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let left = *self as u8;
        let right = *other as u8;
        Some(left.cmp(&right))
    }
}

impl PartialEq for Precedence {
    fn eq(&self, other: &Self) -> bool {
        (*self as u8) == (*other as u8)
    }
}

impl TryFrom<u8> for Precedence {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            value if value == Precedence::None as u8 => Ok(Precedence::None),
            value if value == Precedence::Assignment as u8 => Ok(Precedence::Assignment),
            value if value == Precedence::Or as u8 => Ok(Precedence::Or),
            value if value == Precedence::And as u8 => Ok(Precedence::And),
            value if value == Precedence::Equality as u8 => Ok(Precedence::Equality),
            value if value == Precedence::Comparison as u8 => Ok(Precedence::Comparison),
            value if value == Precedence::Term as u8 => Ok(Precedence::Term),
            value if value == Precedence::Factor as u8 => Ok(Precedence::Factor),
            value if value == Precedence::Unary as u8 => Ok(Precedence::Unary),
            value if value == Precedence::Call as u8 => Ok(Precedence::Call),
            value if value == Precedence::Primary as u8 => Ok(Precedence::Primary),
            _ => Err(())
        }
    }
}

type InfixFn<'a> = fn(&mut Compiler<'a>) -> CompilationResult;
type PrefixFn<'a> = fn(&mut Compiler<'a>, can_assign: bool) -> CompilationResult;

pub enum ParseType<'a> {
    Prefix(PrefixFn<'a>),
    Infix(InfixFn<'a>),
    Both {
        prefix: PrefixFn<'a>,
        infix: InfixFn<'a>,
    },
    None,
}

impl<'a> ParseType<'a> {
    pub fn prefix(&self) -> Option<&PrefixFn<'a>> {
        match self {
            ParseType::Prefix(func) => Some(func),
            ParseType::Both { prefix, infix: _ } => Some(prefix),
            _ => None
        }
    }

    pub fn infix(&self) -> Option<&InfixFn<'a>> {
        match self {
            ParseType::Infix(func) => Some(func),
            ParseType::Both { prefix: _, infix } => Some(infix),
            _ => None
        }
    }
}

pub struct ParseRule<'a> {
    pub parse_type: ParseType<'a>,
    pub precedence: Precedence,
}
