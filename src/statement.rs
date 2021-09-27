use crate::expression::Expression;

#[derive(Debug)]
pub enum Statement {
    Expression(Expression),
    Print(Expression),
    Variable {
        name: String,
        value: Option<Expression>,
    },
}

pub trait Visitor<T> {
    fn visit_print_statement(&self, expression: &Expression) -> T;
    fn visit_expression_statement(&self, expression: &Expression) -> T;
    fn visit_variable_statement(&self, name: &str, value: &Option<Expression>) -> T;
}

impl Statement {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &V) -> T {
        match self {
            Statement::Expression(expr) => visitor.visit_expression_statement(expr),
            Statement::Print(expr) => visitor.visit_print_statement(expr),
            Statement::Variable { name, value } => visitor.visit_variable_statement(name, value),
        }
    }
}
