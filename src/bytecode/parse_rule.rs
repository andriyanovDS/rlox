use super::scanner::ScanError;
use super::compiler::{Compiler, CompileResult};
use super::token::{TokenType, Token};
use std::convert::TryFrom;

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

type ParseFn<'a> = fn(&mut Compiler<'a>) -> CompileResult;

pub enum ParseType<'a> {
    Prefix(ParseFn<'a>),
    Infix(ParseFn<'a>),
    Both {
        prefix: ParseFn<'a>,
        infix: ParseFn<'a>,
    },
    None,
}

impl<'a> ParseType<'a> {
    pub fn prefix(&self) -> Option<&ParseFn<'a>> {
        match self {
            ParseType::Prefix(func) => Some(func),
            ParseType::Both { prefix, infix: _ } => Some(prefix),
            _ => None
        }
    }

    pub fn infix(&self) -> Option<&ParseFn<'a>> {
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
