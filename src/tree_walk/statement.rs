use super::expression::{Expression, VariableExpression};
use super::lox_function::LoxFunction;
use std::rc::Rc;

#[derive(Debug)]
pub enum Statement {
    Expression(Expression),
    Print(Expression),
    Variable {
        name: String,
        value: Option<Expression>,
    },
    Block(Vec<Statement>),
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    Function(Rc<LoxFunction>),
    Return(Expression),
    Class {
        name: String,
        methods: Vec<Rc<LoxFunction>>,
        static_methods: Vec<Rc<LoxFunction>>,
        superclass: Option<VariableExpression>
    }
}

pub trait Visitor<T> {
    fn visit_print(&mut self, expression: &Expression) -> T;
    fn visit_expression(&mut self, expression: &Expression) -> T;
    fn visit_variable_stmt(&mut self, name: &str, value: &Option<Expression>) -> T;
    fn visit_block(&mut self, statements: &[Statement]) -> T;
    fn visit_if(
        &mut self,
        condition: &Expression,
        then_branch: &Statement,
        else_branch: &Option<Box<Statement>>,
    ) -> T;
    fn visit_while(&mut self, condition: &Expression, body: &Statement) -> T;
    fn visit_function(&mut self, func: Rc<LoxFunction>) -> T;
    fn visit_return(&mut self, expression: &Expression) -> T;
    fn visit_class(
        &mut self,
        name: &str,
        methods: &[Rc<LoxFunction>],
        static_methods: &[Rc<LoxFunction>],
        superclass: &Option<VariableExpression>
    ) -> T;
}

impl Statement {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        match self {
            Statement::Expression(expr) => visitor.visit_expression(expr),
            Statement::Print(expr) => visitor.visit_print(expr),
            Statement::Variable { name, value } => visitor.visit_variable_stmt(name, value),
            Statement::Block(statements) => visitor.visit_block(statements),
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => visitor.visit_if(condition, then_branch, else_branch),
            Statement::While { condition, body } => visitor.visit_while(condition, body),
            Statement::Function(func) => visitor.visit_function(func.clone()),
            Statement::Return(expr) => visitor.visit_return(expr),
            Statement::Class { name, methods, static_methods, superclass } => {
                visitor.visit_class(name, methods, static_methods, superclass)
            },
        }
    }
}
