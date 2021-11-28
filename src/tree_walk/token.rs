use super::token_type::TokenType;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: Vec<char>,
    pub line: u32,
    pub id: usize,
}

impl Token {
    pub fn new_single_char(token_type: TokenType, lexeme: char, line: u32, id: usize) -> Token {
        Token {
            token_type,
            lexeme: vec![lexeme],
            line,
            id,
        }
    }

    pub fn new(token_type: TokenType, lexeme: String, line: u32, id: usize) -> Token {
        Token {
            token_type,
            lexeme: lexeme.chars().collect(),
            line,
            id,
        }
    }
}
