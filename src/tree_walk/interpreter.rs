use super::callable::{LoxFn, Callable};
use super::environment::Environment;
use super::error::{Error, InterpreterError};
use super::expression::{self, Expression, LiteralExpression, VariableExpression, Visitor};
use super::lox_function::LoxFunction;
use super::object::Object;
use super::statement::{self, Statement};
use super::token::Token;
use super::token_type::{
    ExpressionOperatorTokenType, KeywordTokenType, SingleCharTokenType, TokenType,
};
use super::lox_class::{CONSTRUCTOR_KEYWORD, LoxClass, SUPER_KEYWORD, THIS_KEYWORD};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::result;

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    pub environment: Rc<RefCell<Environment>>,
    pub locals: HashMap<usize, usize>,
}

pub enum InterpretedValue {
    Some(Object),
    Return(Object),
    None,
}

type StmtInterpretResult = Result<InterpretedValue, InterpreterError>;
type ExprInterpretResult = Result<Object, InterpreterError>;

impl Interpreter {
    pub fn new() -> Self {
        let globals = Interpreter::make_globals();
        let globals = Rc::new(RefCell::new(globals));
        Self {
            globals: globals.clone(),
            environment: globals,
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, statements: &[Statement]) {
        for statement in statements {
            if let Err(error) = statement.accept(self) {
                eprintln!("{}", error.description());
            }
        }
    }

    fn make_globals() -> Environment {
        let mut env = Environment::new();
        env.define("clock".to_string(), Object::make_clock_fn());
        env
    }

    pub fn execute_block(
        &mut self,
        statements: &[Statement],
        environment: Rc<RefCell<Environment>>,
    ) -> StmtInterpretResult {
        let previous_env = self.environment.clone();
        self.environment = environment;
        for statement in statements {
            match statement.accept(self) {
                Err(error) => {
                    self.environment = previous_env;
                    return Err(error);
                },
                Ok(InterpretedValue::Return(stmt)) => {
                    self.environment = previous_env;
                    return Ok(InterpretedValue::Some(stmt));
                }
                _ => {}
            }
        }
        self.environment = previous_env;
        Ok(InterpretedValue::None)
    }
}

impl statement::Visitor<StmtInterpretResult> for Interpreter {
    fn visit_print(&mut self, expression: &Expression) -> StmtInterpretResult {
        match expression.accept(self) {
            Ok(object) => {
                println!("{}", object);
                Ok(InterpretedValue::None)
            }
            Err(err) => Err(err),
        }
    }

    fn visit_expression(&mut self, expression: &Expression) -> StmtInterpretResult {
        expression.accept(self).map(InterpretedValue::Some)
    }

    fn visit_variable_stmt(&mut self, name: &str, value: &Option<Expression>) -> StmtInterpretResult {
        let object = value
            .as_ref()
            .map(|expr| expr.accept(self))
            .unwrap_or(Ok(Object::NotInitialized))?;
        self.environment
            .as_ref()
            .borrow_mut()
            .define(name.to_string(), object);
        Ok(InterpretedValue::None)
    }

    fn visit_block(&mut self, statements: &[Statement]) -> StmtInterpretResult {
        let environment = Environment::from(self.environment.clone());
        self.execute_block(statements, Rc::new(RefCell::new(environment)))
    }

    fn visit_if(
        &mut self,
        condition: &Expression,
        then_branch: &Statement,
        else_branch: &Option<Box<Statement>>,
    ) -> StmtInterpretResult {
        let object = condition.accept(self)?;
        if object.is_truthy() {
            then_branch.accept(self)
        } else {
            else_branch
                .as_ref()
                .map(|stmt| stmt.as_ref().accept(self))
                .unwrap_or(Ok(InterpretedValue::None))
        }
    }

    fn visit_while(&mut self, condition: &Expression, body: &Statement) -> StmtInterpretResult {
        loop {
            let condition = condition.accept(self)?;
            if condition.is_truthy() {
                body.accept(self)?;
            } else {
                return Ok(InterpretedValue::None);
            }
        }
    }

    fn visit_function(&mut self, func: Rc<LoxFunction>) -> StmtInterpretResult {
        let name = func.name.clone();
        let callable = Object::Callable(Callable::LoxFn(LoxFn {
            declaration: func,
            closure: self.environment.clone(),
            is_initializer: false
        }));
        self.environment
            .as_ref()
            .borrow_mut()
            .define(name, callable);
        Ok(InterpretedValue::None)
    }

    fn visit_return(&mut self, expression: &Expression) -> StmtInterpretResult {
        expression.accept(self).map(InterpretedValue::Return)
    }

    fn visit_class(
        &mut self,
        name: &str,
        methods: &[Rc<LoxFunction>],
        static_methods: &[Rc<LoxFunction>],
        superclass: &Option<VariableExpression>
    ) -> StmtInterpretResult {
        let superclass = if let Some(expr) = superclass {
            Some(self.visit_superclass(expr)?)
        } else {
            None
        };

        let mut env = self.environment.as_ref().borrow_mut();
        env.define(name.to_string(), Object::Nil);

        let methods_env = if let Some(expr) = superclass.as_ref() {
            let mut env = Environment::from(self.environment.clone());
            let class_object = Object::Callable(Callable::LoxClass(expr.clone()));
            env.define(SUPER_KEYWORD.to_string(), class_object);
            Rc::new(RefCell::new(env))
        } else {
            self.environment.clone()
        };
        let is_new_env_added = superclass.as_ref().map(|_| true).unwrap_or(false);

        let methods = Interpreter::make_methods_map(&methods_env, methods);
        let static_methods = Interpreter::make_methods_map(&self.environment, static_methods);

        let class = LoxClass {
            name: name.to_string(),
            methods,
            static_methods,
            superclass
        };
        let class_object = Object::Callable(Callable::LoxClass(Rc::new(class)));

        env.assign(name.to_string(), class_object).map_err(|err_msg| {
            InterpreterError::new(0, err_msg) // TODO: pass real line
        })?;
        drop(env);
        if is_new_env_added {
            self.environment = methods_env.as_ref().borrow().enclosing.as_ref().unwrap().clone();
        }
        Ok(InterpretedValue::None)
    }
}

impl expression::Visitor<ExprInterpretResult> for Interpreter {
    fn visit_binary(
        &mut self,
        left: &Expression,
        operator: &Token,
        right: &Expression,
    ) -> ExprInterpretResult {
        let left = left.accept(self)?;
        let right = right.accept(self)?;
        match operator.token_type {
            TokenType::SingleChar(ref token_type) => {
                let result = self.apply_single_char_binary_operation(token_type, &left, &right);
                result.map_err(|message| InterpreterError::new_from_static_str(operator, message))
            }
            TokenType::ExpressionOperator(ref token_type) => {
                let result = self.apply_expression_binary_operation(token_type, &left, &right);
                result.map_err(|message| InterpreterError::new_from_static_str(operator, message))
            }
            _ => Err(InterpreterError::new_from_static_str(
                operator,
                "Unexpected token type",
            )),
        }
    }

    fn visit_grouping(&mut self, expression: &Expression) -> ExprInterpretResult {
        expression.accept(self)
    }

    fn visit_literal(&mut self, literal: &LiteralExpression) -> ExprInterpretResult {
        let object = match literal {
            LiteralExpression::Nil => Object::Nil,
            LiteralExpression::False => Object::Boolean(false),
            LiteralExpression::True => Object::Boolean(true),
            LiteralExpression::Number(num) => Object::Number(*num),
            LiteralExpression::String(str) => Object::String((*str).clone()),
        };
        Ok(object)
    }

    fn visit_unary(&mut self, operator: &Token, right: &Expression) -> ExprInterpretResult {
        let right = right.accept(self)?;
        match (&operator.token_type, right) {
            (&TokenType::SingleChar(SingleCharTokenType::Minus), Object::Number(number)) => {
                Ok(Object::Number(-number))
            }
            (&TokenType::ExpressionOperator(ExpressionOperatorTokenType::Not), object) => {
                Ok(Object::Boolean(!object.is_truthy()))
            }
            _ => Err(InterpreterError::new_from_static_str(
                operator,
                "Operand must be a number",
            )),
        }
    }

    fn visit_variable(&mut self, literal: &str, token: &Token) -> ExprInterpretResult {
        let result = match self.locals.get(&token.id) {
            Some(distance) => self
                .environment
                .borrow()
                .get_at_distance(*distance, literal),
            None => self.globals.as_ref().borrow().get(literal),
        };
        result.map_err(|message| InterpreterError::new(token.line as usize, message))
    }

    fn visit_assignment(&mut self, token: &Token, right: &Expression) -> ExprInterpretResult {
        let object = right.accept(self)?;
        let name: String = token.lexeme.iter().collect();
        let result = match self.locals.get(&token.id) {
            Some(distance) => self.environment.as_ref().borrow_mut().assign_at_distance(
                *distance,
                name,
                object.clone(),
            ),
            None => self
                .globals
                .as_ref()
                .borrow_mut()
                .assign(name, object.clone()),
        };
        result
            .map(|()| object)
            .map_err(|message| InterpreterError::new_from_token(token, message))
    }

    fn visit_logical(
        &mut self,
        left: &Expression,
        operator: &Token,
        right: &Expression,
    ) -> ExprInterpretResult {
        let left = left.accept(self)?;
        match operator.token_type {
            TokenType::Keyword(KeywordTokenType::Or) if left.is_truthy() => Ok(left),
            TokenType::Keyword(KeywordTokenType::And) if !left.is_truthy() => Ok(left),
            _ => right.accept(self),
        }
    }

    fn visit_call(
        &mut self,
        callee: &Expression,
        close_paren: &Token,
        arguments: &[Expression],
    ) -> ExprInterpretResult {
        if let Object::Callable(callable) = &callee.accept(self)? {
            let arg_len = arguments.len();
            let arity = callable.arity();
            if arity != arg_len {
                let message = format!("Expected {} arguments but got {}", arity, arg_len);
                return Err(InterpreterError::new_from_token(close_paren, message));
            }
            let mut obj_arguments = Vec::with_capacity(arg_len);
            for expression in arguments {
                obj_arguments.push(expression.accept(self)?)
            }
            Ok(callable.call(self, &obj_arguments)?)
        } else {
            let error = InterpreterError::new_from_static_str(
                close_paren,
                "Can only call functions and classes.",
            );
            Err(error)
        }
    }

    fn visit_get(&mut self, name: &str, expression: &Expression) -> ExprInterpretResult {
        match expression.accept(self)? {
            Object::Instance(instance) => {
                let borrowed_instance = instance.as_ref().borrow();
                let object = borrowed_instance.get(name, instance.clone()).map_err(|err_msg| {
                    InterpreterError::new(0, err_msg) // TODO: pass real line number
                })?;
                Ok(object)
            },
            Object::Callable(Callable::LoxClass(class)) => {
                let class = class.as_ref();
                let lox_fn = class.find_static_method(name).map_err(|err_msg| {
                    InterpreterError::new(0, err_msg) // TODO: pass real line number
                })?;
                Ok(Object::Callable(Callable::LoxFn(lox_fn.clone())))
            }
            _ => {
                // TODO: pass real line number
                Err(InterpreterError::new(0, "Only instances have properties.".to_string()))
            }
        }
    }

    fn visit_set(&mut self, name: &str, object: &Expression, value: &Expression) -> ExprInterpretResult {
        let object = object.accept(self)?;
        if let Object::Instance(instance) = object {
            let value = value.accept(self)?;
            instance.as_ref().borrow_mut().set(name.to_string(), value.clone());
            Ok(value)
        } else {
            Err(InterpreterError::new(0, "Only instances have fields.".to_string())) // TODO: pass real line number
        }
    }

    fn visit_this(&mut self, token: &Token) -> ExprInterpretResult {
        self.visit_variable("this", token)
    }

    fn visit_super(&mut self, keyword_token: &Token, method: &str) -> ExprInterpretResult {
        let distance = self.locals.get(&keyword_token.id).unwrap();
        let object = self.environment
            .as_ref()
            .borrow()
            .get_at_distance(*distance, SUPER_KEYWORD)
            .map_err(|err_msg| InterpreterError::new_from_token(keyword_token, err_msg))?;
        let instance = self.environment
            .as_ref()
            .borrow()
            .get_at_distance(distance - 1, THIS_KEYWORD)
            .map_err(|err_msg| InterpreterError::new_from_token(keyword_token, err_msg))?;
        let lox_fn = match (object, instance) {
            (Object::Callable(Callable::LoxClass(class)), Object::Instance(instance)) => {
                class.as_ref().find_method(method).map(|method| method.bind(instance.clone()))
            },
            _ => None
        };
        lox_fn
            .map(|lox_fn| Object::Callable(Callable::LoxFn(lox_fn)))
            .ok_or_else(|| {
                InterpreterError::new_from_token(keyword_token, format!("Undefined property {}.", method))
            })
    }
}

impl Interpreter {
    pub fn resolve(&mut self, expression_id: usize, depth: usize) {
        self.locals.insert(expression_id, depth);
    }

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

    fn make_methods_map(env: &Rc<RefCell<Environment>>, methods: &[Rc<LoxFunction>]) -> HashMap<String, LoxFn> {
        methods.into_iter().fold(HashMap::new(), |mut methods, method, | {
            let func = LoxFn {
                declaration: method.clone(),
                closure: env.clone(),
                is_initializer: method.name == CONSTRUCTOR_KEYWORD
            };
            methods.insert(method.name.clone(), func);
            methods
        })
    }

    fn visit_superclass(&mut self, expression: &VariableExpression) -> Result<Rc<LoxClass>, InterpreterError> {
        let superclass = self.visit_variable(&expression.name, &expression.token)?;
        if let Object::Callable(Callable::LoxClass(class)) = superclass {
            Ok(class.clone())
        } else {
            let error = InterpreterError::new_from_token(&expression.token, "Superclass must be a class.".to_string());
            Err(error)
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

impl Callable {
    fn arity(&self) -> usize {
        match self {
            Callable::NativeFn(func) => func.arity,
            Callable::LoxFn(lox_fn) => lox_fn.declaration.arity(),
            Callable::LoxClass(class) => {
                class.find_method(CONSTRUCTOR_KEYWORD)
                    .map(|lox_fn| lox_fn.declaration.arity())
                    .unwrap_or(0)
            },
        }
    }
}
