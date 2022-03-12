use super::Value;

#[derive(Clone)]
pub struct ObjectNativeFunction {
    pub function: Box<fn() -> Value>
}
