use crate::expression::Expression;

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
}

pub trait Visitor<T> {
    fn visit_print_statement(&mut self, expression: &Expression) -> T;
    fn visit_expression_statement(&mut self, expression: &Expression) -> T;
    fn visit_variable_statement(&mut self, name: &str, value: &Option<Expression>) -> T;
    fn visit_block_statement(&mut self, statements: &[Statement]) -> T;
    fn visit_if_statement(
        &mut self,
        condition: &Expression,
        then_branch: &Statement,
        else_branch: &Option<Box<Statement>>,
    ) -> T;
}

impl Statement {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        match self {
            Statement::Expression(expr) => visitor.visit_expression_statement(expr),
            Statement::Print(expr) => visitor.visit_print_statement(expr),
            Statement::Variable { name, value } => visitor.visit_variable_statement(name, value),
            Statement::Block(statements) => visitor.visit_block_statement(statements),
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => visitor.visit_if_statement(condition, then_branch, else_branch),
        }
    }
}
