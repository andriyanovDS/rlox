use crate::interpreter::Interpreter;
use crate::object::Object;

#[derive(Debug, PartialEq, Clone)]
pub struct Callable {}

impl Callable {

    pub fn call(&self, interpreter: &Interpreter, arguments: &[Object]) -> Object {
        todo!()
    }
}
