use crate::error::InterpreterError;
use crate::expression::{self, Expression, LiteralExpression};
use crate::interpreter::Interpreter;
use crate::lox_function::LoxFunction;
use crate::statement::{self, Statement};
use crate::token::Token;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

pub struct Resolver {
    interpreter: Rc<RefCell<Interpreter>>,
    scopes: VecDeque<HashMap<String, VariableState>>,
    current_function_type: FunctionType,
}

#[derive(Copy, Clone, PartialEq)]
enum VariableState {
    Declared,
    Defined,
    Read
}

#[derive(Copy, Clone)]
enum FunctionType {
    None,
    Function
}

type ResolveResult = Result<(), InterpreterError>;

impl statement::Visitor<ResolveResult> for Resolver {
    fn visit_print(&mut self, expression: &Expression) -> ResolveResult {
        self.resolve_expression(expression)?;
        Ok(())
    }

    fn visit_expression(&mut self, expression: &Expression) -> ResolveResult {
        self.resolve_expression(expression)?;
        Ok(())
    }

    fn visit_variable(&mut self, name: &str, value: &Option<Expression>) -> ResolveResult {
        self.declare(name)?;
        if let Some(expression) = value {
            self.resolve_expression(expression)?;
        }
        self.define(name);
        Ok(())
    }

    fn visit_block(&mut self, statements: &[Statement]) -> ResolveResult {
        self.begin_scope();
        self.resolve_statements(statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_if(
        &mut self,
        condition: &Expression,
        then_branch: &Statement,
        else_branch: &Option<Box<Statement>>,
    ) -> ResolveResult {
        self.resolve_expression(condition)?;
        self.resolve_statement(then_branch)?;
        if let Some(else_branch) = else_branch {
            self.resolve_statement(else_branch)?;
        }
        Ok(())
    }

    fn visit_while(&mut self, condition: &Expression, body: &Statement) -> ResolveResult {
        self.resolve_expression(condition)?;
        self.resolve_statement(body)?;
        Ok(())
    }

    fn visit_function(&mut self, func: Rc<LoxFunction>) -> ResolveResult {
        let func = func.as_ref();
        self.declare(&func.name)?;
        self.define(&func.name);
        self.resolve_function(&func.parameters, &func.body, FunctionType::Function)?;
        Ok(())
    }

    fn visit_return(&mut self, expression: &Expression) -> ResolveResult {
        match self.current_function_type {
            FunctionType::None => {
                Err(InterpreterError::new(0, "Can't return from top-level code.".to_string()))
            },
            FunctionType::Function => {
                self.resolve_expression(expression)?;
                Ok(())
            }
        }
    }

    fn visit_class(&mut self, name: &str, _methods: &[Rc<LoxFunction>]) -> ResolveResult {
        self.declare(name)?;
        self.define(name);
        Ok(())
    }
}

impl expression::Visitor<ResolveResult> for Resolver {
    fn visit_binary(
        &mut self,
        left: &Expression,
        _operator: &Token,
        right: &Expression,
    ) -> ResolveResult {
        self.resolve_expression(left)?;
        self.resolve_expression(right)?;
        Ok(())
    }

    fn visit_grouping(&mut self, expression: &Expression) -> ResolveResult {
        self.resolve_expression(expression)?;
        Ok(())
    }

    fn visit_literal(&mut self, _literal: &LiteralExpression) -> ResolveResult {
        Ok(())
    }

    fn visit_unary(&mut self, _operator: &Token, right: &Expression) -> ResolveResult {
        self.resolve_expression(right)?;
        Ok(())
    }

    fn visit_variable(&mut self, literal: &str, token: &Token) -> ResolveResult {
        let current_val = self
            .scopes
            .front()
            .and_then(|v| v.get(literal))
            .copied();

        match current_val {
            Some(VariableState::Declared) => {
                Err(InterpreterError::new_from_static_str(
                    token,
                    "Can't read local variable in its own initializer.",
                ))
            },
            _ => {
                self.resolve_local(literal, token.id, true);
                Ok(())
            }
        }
    }

    fn visit_assignment(&mut self, token: &Token, right: &Expression) -> ResolveResult {
        self.resolve_expression(right)?;
        let variable_name: String = token.lexeme.iter().collect();
        self.resolve_local(&variable_name, token.id, false);
        Ok(())
    }

    fn visit_logical(
        &mut self,
        left: &Expression,
        _operator: &Token,
        right: &Expression,
    ) -> ResolveResult {
        self.resolve_expression(left)?;
        self.resolve_expression(right)?;
        Ok(())
    }

    fn visit_call(
        &mut self,
        callee: &Expression,
        _close_paren: &Token,
        arguments: &[Expression],
    ) -> ResolveResult {
        self.resolve_expression(callee)?;
        for arg in arguments {
            self.resolve_expression(arg)?;
        }
        Ok(())
    }
}

impl Resolver {
    pub fn new(interpreter: Rc<RefCell<Interpreter>>) -> Self {
        Self {
            interpreter,
            scopes: VecDeque::new(),
            current_function_type: FunctionType::None
        }
    }

    pub fn resolve_statements(&mut self, statements: &[Statement]) -> ResolveResult {
        for statement in statements {
            self.resolve_statement(statement)?;
        }
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push_front(HashMap::new())
    }

    fn end_scope(&mut self) {
        if let Some(scope) = self.scopes.pop_front() {
            scope
                .iter()
                .filter(|(_, state)| state != &&VariableState::Read)
                .for_each(|(key, _)| {
                    eprintln!("Local variable {} is not used.", key);
                })
        }
    }

    fn resolve_statement(&mut self, statement: &Statement) -> ResolveResult {
        statement.accept(self)
    }

    fn resolve_expression(&mut self, expression: &Expression) -> ResolveResult {
        expression.accept(self)
    }

    fn declare(&mut self, name: &str) -> ResolveResult {
        match self.scopes.front_mut() {
            Some(inner_scope) if inner_scope.contains_key(name) => {
                let message = "Already a variable with this name in this scope.".to_string();
                // TODO: we need to find a way how to pass a real line number here
                Err(InterpreterError::new(0, message))
            }
            Some(inner_scope) => {
                inner_scope.insert(name.to_string(), VariableState::Declared);
                Ok(())
            }
            None => Ok(()),
        }
    }

    fn define(&mut self, name: &str) {
        let option_ref = self.scopes.front_mut().and_then(|v| v.get_mut(name));
        if let Some(state) = option_ref {
            *state = VariableState::Defined;
        }
    }

    fn resolve_local(&mut self, name: &str, token_id: usize, is_read: bool)  {
        let scope_len = self.scopes.len();
        for index in (0..scope_len).rev() {
            let scope = &mut self.scopes[index];
            if !scope.contains_key(name) {
                continue;
            }
            self.interpreter
                .as_ref()
                .borrow_mut()
                .resolve(token_id, index);

            if is_read {
                scope.insert(name.to_string(), VariableState::Read);
            }
        }
    }

    fn resolve_function(&mut self, params: &[String], body: &[Statement], fn_type: FunctionType) -> ResolveResult {
        let enclosing_function = self.current_function_type;
        self.current_function_type = fn_type;

        self.begin_scope();
        for parameter in params {
            self.declare(parameter)?;
            self.define(parameter);
        }
        self.resolve_statements(body)?;
        self.end_scope();
        self.current_function_type = enclosing_function;

        Ok(())
    }
}
