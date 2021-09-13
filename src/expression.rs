use crate::token::Token;
use std::fmt::{Debug, Formatter};

pub trait Visitor<Result> {
    fn visit_binary(&self, left: &Expression, operator: &Token, right: &Expression) -> Result;
    fn visit_grouping(&self, expression: &Expression) -> Result;
    fn visit_literal(&self, literal: &LiteralExpression) -> Result;
    fn visit_unary(&self, operator: &Token, right: &Expression) -> Result;
    fn visit_variable(&self, literal: String) -> Result;
}

#[derive(Debug)]
pub enum Expression {
    Binary(Box<Expression>, Token, Box<Expression>),
    Grouping(Box<Expression>),
    Literal(LiteralExpression),
    Unary(Token, Box<Expression>),
    Variable(String),
}

#[derive(Debug)]
pub enum LiteralExpression {
    False,
    True,
    Nil,
    String(String),
    Number(f64),
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
            Expression::Variable(ref literal) => visitor.visit_variable(literal.clone()),
        }
    }
}
