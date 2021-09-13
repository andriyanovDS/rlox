use crate::token_type::TokenType;

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: Vec<char>,
    pub line: u32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: Vec<char>, line: u32) -> Token {
        Token {
            token_type,
            lexeme,
            line,
        }
    }

    pub fn new_single_char(token_type: TokenType, lexeme: char, line: u32) -> Token {
        Token {
            token_type,
            lexeme: vec![lexeme],
            line,
        }
    }
}
