use std::fmt::{Debug, Formatter};
use std::cmp::PartialEq;
use std::rc::Rc;

pub mod object_function;
pub mod object_string;
pub mod object_native_function;
pub mod object_closure;
pub mod object_upvalue;

use object_function::ObjectFunction;
use object_string::ObjectString;
use object_native_function::ObjectNativeFunction;
use object_closure::ObjectClosure;

#[derive(Clone)]
pub enum Value {
    Number(f32),
    Bool(bool),
    Nil,
    String(Rc<ObjectString>),
    Function(Rc<ObjectFunction>),
    NativeFunction(ObjectNativeFunction),
    Closure(Rc<ObjectClosure>),
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
            },
            (Value::Function(left), Value::Function(right)) => {
                Rc::as_ptr(left) == Rc::as_ptr(right)
            }
            (Value::NativeFunction(left), Value::NativeFunction(right)) => {
                *(left.function) == *(right.function)
            },
            (Value::Closure(left), Value::Closure(right)) => {
                Rc::as_ptr(left) == Rc::as_ptr(right)
            }
            _ => false
        }
    }
}

impl Debug for Value {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(boolean) => write!(formatter, "{:5}", boolean),
            Value::Number(number) => write!(formatter, "{:5}", number),
            Value::String(object) => write!(formatter, "{:5?}", object.as_ref().value),
            Value::Function(obj) => write!(formatter, "fn<{:?}>", obj.as_ref().name),
            Value::NativeFunction(_) => write!(formatter, "<native fn>"),
            Value::Closure(obj) => write!(formatter, "fn<{:?}>", obj.as_ref().function.name),
            Value::Nil => write!(formatter, "{:5}", "Nil"),
        }
    }
}
