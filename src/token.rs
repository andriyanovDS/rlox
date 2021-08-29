use crate::token_type::TokenType;

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    lexeme: Vec<char>,
    line: u32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: Vec<char>, line: u32) -> Token {
        Token {
            token_type: token_type,
            lexeme: lexeme,
            line: line,
        }
    }

    pub fn new_single_char(token_type: TokenType, lexeme: char, line: u32) -> Token {
        Token {
            token_type: token_type,
            lexeme: vec![lexeme],
            line: line,
        }
    }
}
