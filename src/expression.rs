use crate::token::Token;
use crate::token_type::LiteralTokenType;

pub trait Visitor<Result> {
    fn visit_binary(&self, left: &Expression, operator: &Token, right: &Expression) -> Result;
    fn visit_grouping(&self, expression: &Expression) -> Result;
    fn visit_literal(&self, literal: &LiteralTokenType) -> Result;
    fn visit_unary(&self, operator: &Token, right: &Expression) -> Result;
}

pub enum Expression {
    Binary(Box<Expression>, Token, Box<Expression>),
    Grouping(Box<Expression>),
    Literal(LiteralTokenType),
    Unary(Token, Box<Expression>),
}

impl Expression {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &V) -> T {
        match self {
            Expression::Binary(ref left, ref operator, ref right) => {
                visitor.visit_binary(left, operator, right)
            }
            Expression::Grouping(ref expression) => visitor.visit_grouping(expression),
            Expression::Literal(ref literal) => visitor.visit_literal(literal),
            Expression::Unary(ref operator, ref right) => visitor.visit_unary(operator, right),
        }
    }
}
