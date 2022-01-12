use std::fmt::{Debug, Formatter, Result};
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Clone, Debug)]
pub enum Value {
    Number(f32),
    Bool(bool),
    Nil,
    String(String),
}
