use crate::environment::Environment;
use crate::error::InterpreterError;
use crate::interpreter::Interpreter;
use crate::lox_function::LoxFunction;
use crate::native_function::NativeFunction;
use crate::lox_class::{LoxClass, Instance};
use crate::object::Object;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

#[derive(Clone)]
pub struct LoxFn {
    pub declaration: Rc<LoxFunction>,
    pub closure: Rc<RefCell<Environment>>,
}

#[derive(Clone)]
pub enum Callable {
    LoxFn(LoxFn),
    LoxClass(Rc<LoxClass>),
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
            Callable::LoxFn(lox_fn) => lox_fn.declaration.call(interpreter, arguments, lox_fn.closure.clone()),
            Callable::LoxClass(declaration) => {
                let instance = Instance::new(declaration.clone());
                Ok(Object::Instance(Rc::new(RefCell::new(instance))))
            },
        }
    }
}

impl Debug for Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Callable::LoxFn(lox_fn) => lox_fn.declaration.fmt(f),
            Callable::NativeFn(func) => func.fmt(f),
            Callable::LoxClass(declaration) => declaration.fmt(f)
        }
    }
}

impl LoxFn {
    pub fn bind(&self, instance: Rc<RefCell<Instance>>) -> LoxFn {
        let mut closure = Environment::from(self.closure.clone());
        closure.define("this".to_string(), Object::Instance(instance));
        LoxFn {
            declaration: self.declaration.clone(),
            closure: Rc::new(RefCell::new(closure))
        }
    }
}
