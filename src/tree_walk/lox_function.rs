use super::environment::Environment;
use super::error::InterpreterError;
use super::interpreter::{Interpreter, InterpretedValue};
use super::object::Object;
use super::statement::Statement;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

pub struct LoxFunction {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
}

impl LoxFunction {
    pub fn new(name: String, parameters: Vec<String>, body: Vec<Statement>) -> Self {
        Self {
            name,
            parameters,
            body,
        }
    }

    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[Object],
        closure: Rc<RefCell<Environment>>,
    ) -> Result<Object, InterpreterError> {
        let mut environment = Environment::from(closure);
        for (index, parameter) in self.parameters.iter().enumerate() {
            environment.define(parameter.clone(), arguments[index].clone())
        }
        let result = interpreter.execute_block(&self.body, Rc::new(RefCell::new(environment)))?;
        let object = match result {
            InterpretedValue::None => Object::Nil,
            InterpretedValue::Some(obj)=> obj,
            InterpretedValue::Return(obj) => obj
        };
        Ok(object)
    }

    pub fn arity(&self) -> usize {
        self.parameters.len()
    }
}

impl Debug for LoxFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name)
    }
}
