use crate::token::Token;
use std::fmt::Debug;

pub trait Visitor<Result> {
    fn visit_binary(&self, left: &Expression, operator: &Token, right: &Expression) -> Result;
    fn visit_grouping(&self, expression: &Expression) -> Result;
    fn visit_literal(&self, literal: &LiteralExpression) -> Result;
    fn visit_unary(&self, operator: &Token, right: &Expression) -> Result;
    fn visit_variable(&self, literal: &str, token: &Token) -> Result;
    fn visit_assignment(&self, token: &Token, right: &Expression) -> Result;
    fn visit_logical(&self, left: &Expression, operator: &Token, right: &Expression) -> Result;
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Binary(Box<Expression>, Token, Box<Expression>),
    Grouping(Box<Expression>),
    Literal(LiteralExpression),
    Unary(Token, Box<Expression>),
    Variable { name: String, token: Token },
    Assignment(Token, Box<Expression>),
    Logical(Box<Expression>, Token, Box<Expression>)
}

#[derive(Debug, PartialEq)]
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
            Expression::Binary(left, operator, right) =>
                visitor.visit_binary(left, operator, right),
            Expression::Grouping(expression) => visitor.visit_grouping(expression),
            Expression::Literal(literal) => visitor.visit_literal(literal),
            Expression::Unary(operator, right) => visitor.visit_unary(operator, right),
            Expression::Variable { name, token } => visitor.visit_variable(name, token),
            Expression::Assignment(token,  expr) => visitor.visit_assignment(token, expr),
            Expression::Logical(left, token, right) =>
                visitor.visit_logical(left, token, right),
        }
    }
}
