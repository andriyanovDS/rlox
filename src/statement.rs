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
    While {
        condition: Expression,
        body: Box<Statement>
    }
}

pub trait Visitor<T> {
    fn visit_print(&mut self, expression: &Expression) -> T;
    fn visit_expression(&mut self, expression: &Expression) -> T;
    fn visit_variable(&mut self, name: &str, value: &Option<Expression>) -> T;
    fn visit_block(&mut self, statements: &[Statement]) -> T;
    fn visit_if(
        &mut self,
        condition: &Expression,
        then_branch: &Statement,
        else_branch: &Option<Box<Statement>>,
    ) -> T;
    fn visit_while(&mut self, condition: &Expression, body: &Statement) -> T;
}

impl Statement {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        match self {
            Statement::Expression(expr) => visitor.visit_expression(expr),
            Statement::Print(expr) => visitor.visit_print(expr),
            Statement::Variable { name, value } => visitor.visit_variable(name, value),
            Statement::Block(statements) => visitor.visit_block(statements),
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => visitor.visit_if(condition, then_branch, else_branch),
            Statement::While { condition, body } => visitor.visit_while(condition, body)
        }
    }
}
