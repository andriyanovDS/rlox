#[derive(Copy, Clone, Debug)]
pub enum Value {
    Number(f32),
    Bool(bool),
    Nil,
}
