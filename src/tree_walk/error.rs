use super::token::Token;

pub trait Error {
    fn message(&self) -> &str;
    fn line(&self) -> usize;

    fn description(&self) -> String {
        format!("[line: {}] Error: {}", self.line(), self.message())
    }
}

pub struct InterpreterError {
    line: usize,
    message: String,
}

impl InterpreterError {
    pub fn new(line: usize, message: String) -> Self {
        Self { line, message }
    }

    pub fn new_from_token(token: &Token, message: String) -> Self {
        Self {
            line: token.line as usize,
            message,
        }
    }

    pub fn new_from_static_str(token: &Token, message: &'static str) -> Self {
        Self {
            line: token.line as usize,
            message: message.to_string(),
        }
    }
}

impl Error for InterpreterError {
    fn message(&self) -> &str {
        &self.message
    }
    fn line(&self) -> usize {
        self.line
    }
}
