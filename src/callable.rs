use crate::environment::Environment;
use crate::error::InterpreterError;
use crate::interpreter::Interpreter;
use crate::lox_function::LoxFunction;
use crate::native_function::NativeFunction;
use crate::object::Object;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

#[derive(Clone)]
pub enum Callable {
    LoxFn {
        declaration: Rc<LoxFunction>,
        closure: Rc<RefCell<Environment>>,
    },
    NativeFn(NativeFunction),
}

impl Callable {
    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[Object],
    ) -> Result<Object, InterpreterError> {
        match self {
            Callable::NativeFn(func) => Ok(func.call(arguments)),
            Callable::LoxFn {
                declaration,
                closure,
            } => declaration.call(interpreter, arguments, closure.clone()),
        }
    }
}

impl Debug for Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Callable::LoxFn {
                declaration,
                closure: _,
            } => declaration.fmt(f),
            Callable::NativeFn(func) => func.fmt(f),
        }
    }
}
