use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::cmp::PartialEq;
use std::rc::Rc;

pub mod object_function;
pub mod object_string;
pub mod object_native_function;
pub mod object_closure;
pub mod object_upvalue;
pub mod object_class;
pub mod object_instance;
pub mod object_bound_method;

use object_function::ObjectFunction;
use object_string::ObjectString;
use object_native_function::ObjectNativeFunction;
use object_closure::ObjectClosure;
use object_class::ObjectClass;
use object_instance::ObjectInstance;
use object_bound_method::ObjectBoundMethod;

#[derive(Clone)]
pub enum Value {
    Number(f32),
    Bool(bool),
    Nil,
    String(Rc<ObjectString>),
    Function(Rc<ObjectFunction>),
    NativeFunction(ObjectNativeFunction),
    Closure(Rc<ObjectClosure>),
    Class(Rc<RefCell<ObjectClass>>),
    Instance(Rc<RefCell<ObjectInstance>>),
    BoundMethod(ObjectBoundMethod),
}

impl Default for Value {
    fn default() -> Self { Self::Nil }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::Bool(left), Value::Bool(right)) => left == right,
            (Value::Nil, Value::Nil) => true,
            (Value::String(left), Value::String(right)) => {
                Rc::as_ptr(left) == Rc::as_ptr(right)
            }
            (Value::Function(left), Value::Function(right)) => {
                Rc::as_ptr(left) == Rc::as_ptr(right)
            }
            (Value::NativeFunction(left), Value::NativeFunction(right)) => {
                *(left.function) == *(right.function)
            }
            (Value::Closure(left), Value::Closure(right)) => {
                Rc::as_ptr(left) == Rc::as_ptr(right)
            }
            (Value::Class(left), Value::Class(right)) => {
                Rc::as_ptr(left) == Rc::as_ptr(right)
            }
            (Value::Instance(left), Value::Instance(right)) => {
                Rc::as_ptr(left) == Rc::as_ptr(right)
            }
            (Value::BoundMethod(left), Value::BoundMethod(right)) => left.eq(right),
            _ => false
        }
    }
}

impl Debug for Value {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(boolean) => write!(formatter, "{:5}", boolean),
            Value::Number(number) => write!(formatter, "{:5}", number),
            Value::String(object) => object.fmt(formatter),
            Value::Function(obj) => write!(formatter, "fn<{:?}>", obj.as_ref().name),
            Value::NativeFunction(_) => write!(formatter, "<native fn>"),
            Value::Closure(obj) => obj.as_ref().function.fmt(formatter),
            Value::Class(class) => write!(formatter, "{:?}", class.as_ref().borrow().name),
            Value::Instance(instance) => {
                write!(formatter, "{:?} instance", instance.as_ref().borrow().class.as_ref().borrow().name)
            },
            Value::Nil => write!(formatter, "{:5}", "Nil"),
            Value::BoundMethod(method) => method.fmt(formatter),
        }
    }
}
