use crate::callable::Callable;
use crate::native_function::NativeFunction;
use crate::object::Object;
use std::time::{SystemTime, UNIX_EPOCH};

impl Object {
    pub fn make_clock_fn() -> Object {
        let native_fn = NativeFunction {
            arity: 0,
            on_call: Box::new(|_| {
                let system_time = SystemTime::now();
                let milliseconds = system_time.duration_since(UNIX_EPOCH).unwrap().as_millis();
                Object::Number(milliseconds as f64)
            }),
        };
        Object::Callable(Callable::NativeFn(native_fn))
    }
}
