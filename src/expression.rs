use crate::token::Token;
use std::fmt::Debug;

pub trait Visitor<Result> {
    fn visit_binary(&mut self, left: &Expression, operator: &Token, right: &Expression) -> Result;
    fn visit_grouping(&mut self, expression: &Expression) -> Result;
    fn visit_literal(&mut self, literal: &LiteralExpression) -> Result;
    fn visit_unary(&mut self, operator: &Token, right: &Expression) -> Result;
    fn visit_variable(&mut self, literal: &str, token: &Token) -> Result;
    fn visit_assignment(&mut self, token: &Token, right: &Expression) -> Result;
    fn visit_logical(&mut self, left: &Expression, operator: &Token, right: &Expression) -> Result;
    fn visit_call(
        &mut self,
        callee: &Expression,
        close_paren: &Token,
        arguments: &[Expression],
    ) -> Result;
    fn visit_get(&mut self, name: &str, expression: &Expression) -> Result;
    fn visit_set(&mut self, name: &str, object: &Expression, value: &Expression) -> Result;
    fn visit_this(&mut self, token: &Token) -> Result;
    fn visit_super(&mut self, keyword_token: &Token, method: &str) -> Result;
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Binary(Box<Expression>, Token, Box<Expression>),
    Grouping(Box<Expression>),
    Literal(LiteralExpression),
    Unary(Token, Box<Expression>),
    Variable(VariableExpression),
    Assignment(Token, Box<Expression>),
    Logical(Box<Expression>, Token, Box<Expression>),
    Call {
        callee: Box<Expression>,
        close_paren: Token,
        arguments: Vec<Expression>,
    },
    Get {
        name: String,
        expression: Box<Expression>
    },
    Set {
        name: String,
        object: Box<Expression>,
        value: Box<Expression>
    },
    This(Token),
    Super {
        keyword_token: Token,
        method: String
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralExpression {
    False,
    True,
    Nil,
    String(String),
    Number(f64),
}

#[derive(Debug, PartialEq, Clone)]
pub struct VariableExpression {
    pub name: String,
    pub token: Token,
}

impl Expression {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        match self {
            Expression::Binary(left, operator, right) => {
                visitor.visit_binary(left, operator, right)
            }
            Expression::Grouping(expression) => visitor.visit_grouping(expression),
            Expression::Literal(literal) => visitor.visit_literal(literal),
            Expression::Unary(operator, right) => visitor.visit_unary(operator, right),
            Expression::Variable(expr) => visitor.visit_variable(&expr.name, &expr.token),
            Expression::Assignment(token, expr) => visitor.visit_assignment(token, expr),
            Expression::Logical(left, token, right) => visitor.visit_logical(left, token, right),
            Expression::Call {
                callee,
                close_paren,
                arguments,
            } => visitor.visit_call(callee, close_paren, arguments),
            Expression::Get { name, expression } => visitor.visit_get(name, expression),
            Expression::Set { name, object, value } => visitor.visit_set(name, object, value),
            Expression::This(token) => visitor.visit_this(token),
            Expression::Super { keyword_token, method } => visitor.visit_super(keyword_token, method),
        }
    }
}
