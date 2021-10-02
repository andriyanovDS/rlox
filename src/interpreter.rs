use crate::environment::Environment;
use crate::expression::{self, Expression, LiteralExpression};
use crate::object::Object;
use crate::statement::{self, Statement};
use crate::token::Token;
use crate::token_type::{ExpressionOperatorTokenType, SingleCharTokenType, TokenType};
use std::cell::RefCell;
use std::rc::Rc;
use std::result;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

struct InterpretError {
    line: u32,
    message: String,
}

type StmtInterpretResult = Result<(), InterpretError>;
type ExprInterpretResult = Result<Object, InterpretError>;

impl InterpretError {
    fn new(token: &Token, message: &'static str) -> Self {
        Self {
            line: token.line,
            message: message.to_string(),
        }
    }
    fn to_error_message(&self) -> String {
        format!("[line: {}] {}", self.line, self.message)
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, statements: &[Statement]) {
        for statement in statements {
            if let Err(error) = statement.accept(self) {
                eprintln!("{}", error.to_error_message());
            }
        }
    }
}

impl statement::Visitor<StmtInterpretResult> for Interpreter {
    fn visit_print_statement(&mut self, expression: &Expression) -> StmtInterpretResult {
        match expression.accept(self) {
            Ok(object) => {
                println!("{}", object);
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn visit_expression_statement(
        &mut self,
        expression: &Expression,
    ) -> StmtInterpretResult {
        expression.accept(self).map(|_| ())
    }

    fn visit_variable_statement(
        &mut self,
        name: &str,
        value: &Option<Expression>,
    ) -> StmtInterpretResult {
        let object = value
            .as_ref()
            .map(|expr| expr.accept(self))
            .unwrap_or(Ok(Object::NotInitialized))?;
        self.environment
            .borrow_mut()
            .define(name.to_string(), object);
        Ok(())
    }

    fn visit_block_statement(&mut self, statements: &[Statement]) -> StmtInterpretResult {
        let previous_env = self.environment.clone();
        let environment = Environment::from(self.environment.clone());
        self.environment = Rc::new(RefCell::new(environment));
        for statement in statements {
            if let Err(error) = statement.accept(self) {
                self.environment = previous_env;
                return Err(error);
            }
        }
        self.environment = previous_env;
        Ok(())
    }
}

impl expression::Visitor<ExprInterpretResult> for Interpreter {
    fn visit_binary(&self, left: &Expression, operator: &Token, right: &Expression) -> ExprInterpretResult {
        let left = left.accept(self)?;
        let right = right.accept(self)?;
        match operator.token_type {
            TokenType::SingleChar(ref token_type) => {
                let result = self.apply_single_char_binary_operation(token_type, &left, &right);
                result.map_err(|message| InterpretError::new(operator, message))
            }
            TokenType::ExpressionOperator(ref token_type) => {
                let result = self.apply_expression_binary_operation(token_type, &left, &right);
                result.map_err(|message| InterpretError::new(operator, message))
            }
            _ => Err(InterpretError::new(operator, "Unexpected token type")),
        }
    }

    fn visit_grouping(&self, expression: &Expression) -> ExprInterpretResult {
        expression.accept(self)
    }

    fn visit_literal(&self, literal: &LiteralExpression) -> ExprInterpretResult {
        let object = match literal {
            LiteralExpression::Nil => Object::Nil,
            LiteralExpression::False => Object::Boolean(false),
            LiteralExpression::True => Object::Boolean(true),
            LiteralExpression::Number(num) => Object::Number(*num),
            LiteralExpression::String(str) => Object::String((*str).clone()),
        };
        Ok(object)
    }

    fn visit_unary(&self, operator: &Token, right: &Expression) -> ExprInterpretResult {
        let right = right.accept(self)?;
        match (&operator.token_type, right) {
            (&TokenType::SingleChar(SingleCharTokenType::Minus), Object::Number(number)) => {
                Ok(Object::Number(-number))
            }
            (&TokenType::ExpressionOperator(ExpressionOperatorTokenType::Not), object) => {
                Ok(Object::Boolean(!object.is_truthy()))
            }
            _ => Err(InterpretError::new(operator, "Operand must be a number")),
        }
    }

    fn visit_variable(&self, literal: &str, token: &Token) -> ExprInterpretResult {
        self.environment
            .borrow()
            .get(literal)
            .map(|object| object.clone())
            .map_err(|message| InterpretError { line: token.line, message })
    }

    fn visit_assignment(&self, token: &Token, right: &Expression) -> ExprInterpretResult {
        let object = right.accept(self)?;
        let name: String = token.lexeme.iter().collect();
        self.environment.borrow_mut().assign(name, object.clone())
            .map(|()| object)
            .map_err(|message| InterpretError { line: token.line, message })
    }
}

impl Interpreter {
    fn apply_single_char_binary_operation(
        &self,
        single_char_token_type: &SingleCharTokenType,
        left: &Object,
        right: &Object,
    ) -> result::Result<Object, &'static str> {
        match (single_char_token_type, left, right) {
            (SingleCharTokenType::Minus, Object::Number(left), Object::Number(right)) => {
                Ok(Object::Number(left - right))
            }
            (SingleCharTokenType::Slash, Object::Number(left), Object::Number(right)) => {
                if *right == 0f64 {
                    Err("Division by zero")
                } else {
                    Ok(Object::Number(left / right))
                }
            }
            (SingleCharTokenType::Star, Object::Number(left), Object::Number(right)) => {
                Ok(Object::Number(left * right))
            }
            (SingleCharTokenType::Plus, Object::Number(left), Object::Number(right)) => {
                Ok(Object::Number(left + right))
            }
            (SingleCharTokenType::Plus, Object::String(left), right) => {
                Ok(Object::String(format!("{}{}", left, right)))
            }
            (SingleCharTokenType::Plus, left, Object::String(right)) => {
                Ok(Object::String(format!("{}{}", left, right)))
            }
            (SingleCharTokenType::Plus, _, _) => Err("Operands must be two numbers or two strings"),
            _ => Err("Operands must be numbers."),
        }
    }

    fn apply_expression_binary_operation(
        &self,
        expr_token_type: &ExpressionOperatorTokenType,
        left: &Object,
        right: &Object,
    ) -> result::Result<Object, &'static str> {
        match (expr_token_type, left, right) {
            (ExpressionOperatorTokenType::Greater, Object::Number(left), Object::Number(right)) => {
                Ok(Object::Boolean(left > right))
            }
            (
                ExpressionOperatorTokenType::GreaterEqual,
                Object::Number(left),
                Object::Number(right),
            ) => Ok(Object::Boolean(left >= right)),
            (ExpressionOperatorTokenType::Less, Object::Number(left), Object::Number(right)) => {
                Ok(Object::Boolean(left < right))
            }
            (
                ExpressionOperatorTokenType::LessEqual,
                Object::Number(left),
                Object::Number(right),
            ) => Ok(Object::Boolean(left <= right)),
            (ExpressionOperatorTokenType::EqualEqual, left, right) => {
                Ok(Object::Boolean(left.is_equal(right)))
            }
            (ExpressionOperatorTokenType::NotEqual, left, right) => {
                Ok(Object::Boolean(!left.is_equal(right)))
            }
            _ => Err("Operands must be numbers."),
        }
    }
}

impl Object {
    fn is_truthy(&self) -> bool {
        match self {
            Object::Boolean(value) => *value,
            Object::Nil => false,
            _ => true,
        }
    }

    fn is_equal(&self, to_object: &Object) -> bool {
        match (self, to_object) {
            (Object::Nil, Object::Nil) => true,
            (Object::Number(left), Object::Number(right)) => (left - right).abs() == 0f64,
            (Object::String(left), Object::String(right)) => left.eq(right),
            (Object::Boolean(left), Object::Boolean(right)) => left == right,
            _ => false,
        }
    }
}
