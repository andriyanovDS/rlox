#[derive(Debug)]
pub enum Object {
    Nil,
    Boolean(bool),
    String(String),
    Number(f64),
}
