use crate::interpreter::{InterpretError, Interpreter};
use crate::lox_function::LoxFunction;
use crate::native_function::NativeFunction;
use crate::object::Object;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

#[derive(Clone)]
pub enum Callable {
    LoxFn(Rc<LoxFunction>),
    NativeFn(NativeFunction),
}

impl Callable {
    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[Object],
    ) -> Result<Object, InterpretError> {
        match self {
            Callable::NativeFn(func) => Ok(func.call(arguments)),
            Callable::LoxFn(func) => func.call(interpreter, arguments),
        }
    }
}

impl Debug for Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Callable::LoxFn(func) => func.fmt(f),
            Callable::NativeFn(func) => func.fmt(f),
        }
    }
}
