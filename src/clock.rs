use crate::object::Object;
use crate::function::Callable;
use std::time::{SystemTime, UNIX_EPOCH};

impl Object {
    pub fn make_clock_fn() -> Object {
        let callable = Callable {
            arity: 0,
            on_call: Box::new(|_| {
                let system_time = SystemTime::now();
                let milliseconds = system_time.duration_since(UNIX_EPOCH).unwrap().as_millis();
                Object::Number(milliseconds as f64)
            })
        };
        Object::Callable(callable)
    }
}
