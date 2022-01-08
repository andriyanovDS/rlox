use std::str::Chars;
use peekmore::{PeekMore, PeekMoreIterator};
use crate::bytecode::token::{Lexeme, TokenType};
use super::token::Token;

pub struct Scanner<'a> {
    source_iter: PeekMoreIterator<Chars<'a>>,
    source: &'a str,
    line: usize,
    token_start_position: usize,
}

pub struct ScanError {
    pub message: &'static str,
    pub line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source_iter: source.chars().peekmore(),
            source,
            line: 1,
            token_start_position: 0,
        }
    }

    pub fn scan_token(&mut self) -> Result<Token, ScanError> {
        self.token_start_position += self.skip_whitespaces();
        match self.source_iter.next() {
            Some(character) => {
                let (token_type, length) = self.scan_token_type(&character)?;
                let mut start_position = self.token_start_position;
                if token_type == TokenType::String {
                    self.token_start_position += length + 2;
                    start_position += 1;
                } else {
                    self.token_start_position += length;
                }
                Ok(self.make_token(token_type, start_position, length))
            },
            None => Ok(Token {
                token_type: TokenType::Eof,
                lexeme: None,
                line: self.line
            })
        }
    }

    fn make_token(&self, token_type: TokenType, start_position: usize, lexeme_length: usize) -> Token {
        Token {
            token_type,
            lexeme: Some(Lexeme { start: start_position, length: lexeme_length }),
            line: self.line
        }
    }

    fn make_error(&self, message: &'static str) -> ScanError {
        ScanError {
            message,
            line: self.line
        }
    }

    fn scan_token_type(&mut self, first_character: &char) -> Result<(TokenType, usize), ScanError> {
        match first_character {
            '(' => Ok((TokenType::LeftParen, 1)),
            ')' => Ok((TokenType::RightParen, 1)),
            '{' => Ok((TokenType::LeftBrace, 1)),
            '}' => Ok((TokenType::RightBrace, 1)),
            ';' => Ok((TokenType::Semicolon, 1)),
            ',' => Ok((TokenType::Comma, 1)),
            '.' => Ok((TokenType::Dot, 1)),
            '-' => Ok((TokenType::Minus, 1)),
            '+' => Ok((TokenType::Plus, 1)),
            '/' => Ok((TokenType::Slash, 1)),
            '*' => Ok((TokenType::Star, 1)),
            '!' => Ok(self.match_token_type('=', || TokenType::BangEqual, || TokenType::Bang)),
            '=' => Ok(self.match_token_type('=', || TokenType::EqualEqual, || TokenType::Equal)),
            '<' => Ok(self.match_token_type('=', || TokenType::LessEqual, || TokenType::Less)),
            '>' => Ok(self.match_token_type('=', || TokenType::GreaterEqual, || TokenType::Greater)),
            '"' => {
                self.consume_literal()
                    .map(|size| (TokenType::String, size))
                    .ok_or(self.make_error("Unterminated string."))
            },
            character if character.is_digit(10) => Ok((TokenType::Number, self.consume_number())),
            character if character.is_alphanumeric() => {
                let length = self.consume_identifier();
                let keyword = &self.source[self.token_start_position..self.token_start_position + length];
                Ok((self.identifier_type(keyword), length))
            },
            _ => Err(self.make_error("Unexpected character."))
        }
    }

    fn match_token_type<F, P>(
        &mut self,
        character: char,
        token_type_provider: F,
        fallback_provider: P
    ) -> (TokenType, usize) where F: FnOnce() -> TokenType, P: FnOnce() -> TokenType  {
        match self.source_iter.peek() {
            Some(next_char) if *next_char == character => {
                self.source_iter.next();
                (token_type_provider(), 2)
            },
            _ => (fallback_provider(), 1)
        }
    }

    fn skip_whitespaces(&mut self) -> usize {
        let mut skipped: usize = 0;
        loop {
            match self.source_iter.peek() {
                Some(' ' | '\r' | '\t') => {
                    self.source_iter.next();
                    skipped += 1;
                },
                Some('\n') => {
                    self.source_iter.next();
                    self.line += 1;
                    skipped += 1;
                },
                Some('/') => {
                    self.source_iter.advance_cursor();
                    if let Some('/') = self.source_iter.peek() {
                        self.source_iter.reset_cursor();
                        skipped += self.skip_comment();
                    } else {
                        self.source_iter.reset_cursor();
                        break skipped;
                    }
                }
                _ => {
                    break skipped;
                }
            }
        }
    }

    fn skip_comment(&mut self) -> usize {
        let mut skipped: usize = 0;
        loop {
            match self.source_iter.peek() {
                Some('\n') => {
                    return skipped
                },
                None => {
                    return skipped
                },
                _ => {
                    self.source_iter.next();
                    skipped += 1;
                }
            }
        }
    }

    fn consume_literal(&mut self) -> Option<usize> {
        let mut length: usize = 0;
        loop {
            match self.source_iter.next() {
                Some('"') => {
                    return Some(length);
                },
                None => {
                    return None;
                },
                Some('\n') => {
                    self.line += 1;
                    length += 1;
                },
                _ => {
                    length += 1;
                }
            }
        }
    }

    fn consume_number(&mut self) -> usize {
        let mut length: usize = 1;
        loop {
            match self.source_iter.peek() {
                Some(character) if character.is_digit(10) => {
                    length += 1;
                    self.source_iter.next();
                },
                Some('.') => {
                    self.source_iter.advance_cursor();
                    if let Some(character) = self.source_iter.peek() {
                        if character.is_digit(10) {
                            length += 1;
                            self.source_iter.reset_cursor();
                            self.source_iter.next();
                            continue;
                        }
                    }
                    return length;
                },
                _ => {
                    return length;
                }
            }
        }
    }

    fn consume_identifier(&mut self) -> usize {
        let mut length: usize = 1;
        loop {
            match self.source_iter.peek() {
                Some(character) if character.is_alphanumeric() => {
                    length += 1;
                    self.source_iter.next();
                },
                _ => {
                    return length;
                }
            }
        }
    }

    fn identifier_type(&self, keyword: &'a str) -> TokenType {
        assert_eq!(keyword.is_empty(), false);
        let mut chars = keyword.chars();
        match chars.next().unwrap() {
            'a' => Scanner::check_keyword(&keyword[1..], "nd", TokenType::And),
            'c' => Scanner::check_keyword(&keyword[1..], "lass", TokenType::Class),
            'e' => Scanner::check_keyword(&keyword[1..], "lse", TokenType::Else),
            'i' => Scanner::check_keyword(&keyword[1..], "f", TokenType::If),
            'n' => Scanner::check_keyword(&keyword[1..], "il", TokenType::Nil),
            'o' => Scanner::check_keyword(&keyword[1..], "r", TokenType::Or),
            'p' => Scanner::check_keyword(&keyword[1..], "rint", TokenType::Print),
            'r' => Scanner::check_keyword(&keyword[1..], "eturn", TokenType::Return),
            's' => Scanner::check_keyword(&keyword[1..], "uper", TokenType::Super),
            'v' => Scanner::check_keyword(&keyword[1..], "ar", TokenType::Var),
            'w' => Scanner::check_keyword(&keyword[1..], "hile", TokenType::While),
            'f' if keyword.len() > 1 => {
                match chars.next().unwrap() {
                    'a' => Scanner::check_keyword(&keyword[2..], "lse", TokenType::False),
                    'o' => Scanner::check_keyword(&keyword[2..], "r", TokenType::For),
                    'u' => Scanner::check_keyword(&keyword[2..], "n", TokenType::Fun),
                    _ => TokenType::Identifier
                }
            },
            't' if keyword.len() > 1 => {
                match chars.next().unwrap() {
                    'r' => Scanner::check_keyword(&keyword[2..], "ue", TokenType::True),
                    'h' => Scanner::check_keyword(&keyword[2..], "is", TokenType::This),
                    _ => TokenType::Identifier
                }
            }
            _ => TokenType::Identifier
        }
    }

    fn check_keyword(left: &str, right: &'static str, token_type: TokenType) -> TokenType {
        if left == right {
            token_type
        } else {
            TokenType::Identifier
        }
    }
}

