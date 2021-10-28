use crate::environment::Environment;
use crate::interpreter::{InterpretError, Interpreter};
use crate::object::Object;
use crate::statement::Statement;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

pub struct LoxFunction {
    pub name: String,
    parameters: Vec<String>,
    body: Vec<Statement>,
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
    ) -> Result<Object, InterpretError> {
        let mut environment = Environment::from(closure);
        for (index, parameter) in self.parameters.iter().enumerate() {
            environment.define(parameter.clone(), arguments[index].clone())
        }
        let result = interpreter.execute_block(&self.body, Rc::new(RefCell::new(environment)))?;
        Ok(result.unwrap_or(Object::Nil))
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
