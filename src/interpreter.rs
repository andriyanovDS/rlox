use crate::expression::{Expression, LiteralExpression, Visitor};
use crate::object::Object;
use crate::token::Token;
use crate::token_type::{ExpressionOperatorTokenType, SingleCharTokenType, TokenType};
use std::result::Result;

struct Interpreter;

struct InterpretError {
    line: u32,
    message: &'static str,
}

impl InterpretError {
    fn new(token: &Token, message: &'static str) -> Self {
        Self { line: token.line, message }
    }

    fn to_error_message(&self) -> String {
        format!("[line: {}] {}", self.line, self.message)
    }
}

type Res = Result<Object, InterpretError>;

impl Interpreter {
    fn interpret(&self, expression: Expression) {
        match expression.accept(self) {
            Ok(object) => println!("{:?}", object),
            Err(error) => eprintln!("{}", error.to_error_message())
        }
    }
}

impl Visitor<Res> for Interpreter {
    fn visit_binary(&self, left: &Expression, operator: &Token, right: &Expression) -> Res {
        let left = left.accept(self)?;
        let right = right.accept(self)?;
        match &operator.token_type {
            &TokenType::SingleChar(ref token_type) => {
                let result = self.apply_single_char_binary_operation(token_type, &left, &right);
                result.map_err(|message| InterpretError::new(operator, message))
            }
            &TokenType::ExpressionOperator(ref token_type) => {
                let result = self.apply_expression_binary_operation(token_type, &left, &right);
                result.map_err(|message| InterpretError::new(operator, message))
            }
            _ => Err(InterpretError::new(operator, "Unexpected token type")),
        }
    }

    fn visit_grouping(&self, expression: &Expression) -> Res {
        Ok(expression.accept(self)?)
    }

    fn visit_literal(&self, literal: &LiteralExpression) -> Res {
        let object = match literal {
            LiteralExpression::Nil => Object::Nil,
            LiteralExpression::False => Object::Boolean(false),
            LiteralExpression::True => Object::Boolean(true),
            LiteralExpression::Number(num) => Object::Number(*num),
            LiteralExpression::String(str) => Object::String((*str).clone()),
        };
        Ok(object)
    }

    fn visit_unary(&self, operator: &Token, right: &Expression) -> Res {
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

    fn visit_variable(&self, literal: String) -> Res {
        Err(InterpretError { line: 0, message: "Not implemented yet" })
    }
}

impl Interpreter {

    fn apply_single_char_binary_operation(
        &self,
        single_char_token_type: &SingleCharTokenType,
        left: &Object,
        right: &Object,
    ) -> Result<Object, &'static str> {
        match (single_char_token_type, left, right) {
            (SingleCharTokenType::Minus, Object::Number(left), Object::Number(right)) => {
                Ok(Object::Number(left - right))
            }
            (SingleCharTokenType::Slash, Object::Number(left), Object::Number(right)) => {
                Ok(Object::Number(left / right))
            }
            (SingleCharTokenType::Star, Object::Number(left), Object::Number(right)) => {
                Ok(Object::Number(left * right))
            }
            (SingleCharTokenType::Plus, Object::Number(left), Object::Number(right)) => {
                Ok(Object::Number(left + right))
            }
            (SingleCharTokenType::Plus, Object::String(left), Object::String(right)) => {
                let mut result_string = left.clone();
                result_string.push_str(&right);
                Ok(Object::String(result_string))
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
    ) -> Result<Object, &'static str> {
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
            (ExpressionOperatorTokenType::Equal, left, right) => {
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
            (Object::Number(left), Object::Number(right)) => left == right,
            (Object::String(left), Object::String(right)) => left.eq(right),
            (Object::Boolean(left), Object::Boolean(right)) => left == right,
            _ => false,
        }
    }
}
