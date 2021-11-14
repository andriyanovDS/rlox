use crate::environment::Environment;
use crate::error::InterpreterError;
use crate::interpreter::Interpreter;
use crate::lox_function::LoxFunction;
use crate::native_function::NativeFunction;
use crate::lox_class::{LoxClass, Instance, CONSTRUCTOR_KEYWORD};
use crate::object::Object;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use crate::lox_class::THIS_KEYWORD;

#[derive(Clone)]
pub struct LoxFn {
    pub declaration: Rc<LoxFunction>,
    pub closure: Rc<RefCell<Environment>>,
    pub is_initializer: bool
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
            Callable::LoxFn(lox_fn) => {
                let result = lox_fn.declaration.call(interpreter, arguments, lox_fn.closure.clone());
                if lox_fn.is_initializer {
                    lox_fn.closure.as_ref().borrow().get(THIS_KEYWORD).map_err(|err_msg| {
                        InterpreterError::new(0, err_msg) // TODO: pass real line number
                    })
                } else {
                    result
                }
            },
            Callable::LoxClass(declaration) => {
                let instance = Instance::new(declaration.clone());
                let rc_instance = Rc::new(RefCell::new(instance));
                let initializer = rc_instance
                    .as_ref()
                    .borrow()
                    .find_method(CONSTRUCTOR_KEYWORD, rc_instance.clone());
                if let Some(Object::Callable(Callable::LoxFn(func))) = initializer {
                    let func = func.bind(rc_instance.clone());
                    func.declaration.call(interpreter, arguments, func.closure.clone())?;
                }
                Ok(Object::Instance(rc_instance))
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
            closure: Rc::new(RefCell::new(closure)),
            is_initializer: self.is_initializer
        }
    }
}
