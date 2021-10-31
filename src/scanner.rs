use crate::token::Token;
use crate::token_type::{
    Delimiter, ExpressionOperatorTokenType, KeywordTokenType, LiteralTokenType,
    SingleCharTokenType, TokenType,
};
use peekmore::{PeekMore, PeekMoreIterator};
use std::collections::HashMap;
use std::str::Chars;

pub struct Scanner<'a> {
    source_iter: PeekMoreIterator<Chars<'a>>,
    current_id: usize,
}

struct MatchedExpression {
    token_type: ExpressionOperatorTokenType,
    lexeme: Vec<char>,
}

impl MatchedExpression {
    fn make_token(self, line: u32, id: usize) -> Token {
        Token {
            token_type: TokenType::ExpressionOperator(self.token_type),
            lexeme: self.lexeme,
            line,
            id,
        }
    }
}

enum CharacterScanResult {
    Token(Token),
    StringLiteral(Token, u32),
    NewLine,
    Skipped,
    Err(String),
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Scanner<'a> {
        Scanner {
            source_iter: source.chars().peekmore(),
            current_id: 0,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens = vec![];
        let mut line = 1u32;
        let keywords = KeywordTokenType::make_keywords();

        while let Some(character) = self.source_iter.next() {
            match self.scan_token(character, line, &keywords) {
                CharacterScanResult::NewLine => {
                    line += 1;
                }
                CharacterScanResult::Token(token) => {
                    tokens.push(token);
                }
                CharacterScanResult::StringLiteral(token, new_line_count) => {
                    line += new_line_count;
                    tokens.push(token);
                }
                CharacterScanResult::Err(message) => {
                    println!("[Line {}] Error: {}", line, message);
                }
                CharacterScanResult::Skipped => {}
            }
        }
        tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: Vec::new(),
            line,
            id: self.make_token_id(),
        });
        tokens
    }

    fn scan_token(
        &mut self,
        character: char,
        line: u32,
        keywords: &HashMap<String, KeywordTokenType>,
    ) -> CharacterScanResult {
        let make_result = |token| CharacterScanResult::Token(token);
        let id = self.make_token_id();
        let make_token =
            |token_type| make_result(Token::new_single_char(token_type, character, line, id));

        match character {
            '(' => make_token(TokenType::OpenDelimiter(Delimiter::Paren)),
            ')' => make_token(TokenType::CloseDelimiter(Delimiter::Paren)),
            '{' => make_token(TokenType::OpenDelimiter(Delimiter::Brace)),
            '}' => make_token(TokenType::CloseDelimiter(Delimiter::Brace)),
            '[' => make_token(TokenType::OpenDelimiter(Delimiter::Bracket)),
            ']' => make_token(TokenType::CloseDelimiter(Delimiter::Bracket)),
            ',' => make_token(TokenType::SingleChar(SingleCharTokenType::Comma)),
            '.' => make_token(TokenType::SingleChar(SingleCharTokenType::Dot)),
            '-' => make_token(TokenType::SingleChar(SingleCharTokenType::Minus)),
            '+' => make_token(TokenType::SingleChar(SingleCharTokenType::Plus)),
            ';' => make_token(TokenType::SingleChar(SingleCharTokenType::Semicolon)),
            '*' => make_token(TokenType::SingleChar(SingleCharTokenType::Star)),
            '!' => make_result(
                self.matches_expression(
                    '!',
                    &'=',
                    ExpressionOperatorTokenType::NotEqual,
                    ExpressionOperatorTokenType::Not,
                )
                .make_token(line, id),
            ),
            '=' => make_result(
                self.matches_expression(
                    '=',
                    &'=',
                    ExpressionOperatorTokenType::EqualEqual,
                    ExpressionOperatorTokenType::Equal,
                )
                .make_token(line, id),
            ),
            '>' => make_result(
                self.matches_expression(
                    '>',
                    &'=',
                    ExpressionOperatorTokenType::GreaterEqual,
                    ExpressionOperatorTokenType::Greater,
                )
                .make_token(line, id),
            ),
            '<' => make_result(
                self.matches_expression(
                    '<',
                    &'=',
                    ExpressionOperatorTokenType::LessEqual,
                    ExpressionOperatorTokenType::Less,
                )
                .make_token(line, id),
            ),
            '/' => {
                if let Some(token_type) = self.scan_slash() {
                    make_token(token_type)
                } else {
                    CharacterScanResult::NewLine
                }
            }
            ' ' | '\r' | '\t' => CharacterScanResult::Skipped,
            '"' => self.scan_string_literal().map_or(
                CharacterScanResult::Err("Unterminated string".to_string()),
                |(literal, line_count)| {
                    let lexeme = literal.chars().clone().collect();
                    let token_type = TokenType::Literal(LiteralTokenType::String(literal));
                    let token = Token {
                        token_type,
                        lexeme,
                        line,
                        id,
                    };
                    CharacterScanResult::StringLiteral(token, line_count)
                },
            ),
            '\n' => CharacterScanResult::NewLine,
            character if character.is_digit(10) => {
                let (number, lexeme) = self.scan_number(character);
                let token_type = TokenType::Literal(LiteralTokenType::Number(number));
                let token = Token::new(token_type, lexeme, line, id);
                make_result(token)
            }
            character if character.is_alphanumeric() => {
                let (token_type, lexeme) = self.scan_identifier(character, keywords);
                let token = Token::new(token_type, lexeme, line, id);
                make_result(token)
            }
            _ => CharacterScanResult::Err(format!("Unknown symbol {}", character)),
        }
    }

    fn make_token_id(&mut self) -> usize {
        let id = self.current_id;
        self.current_id += 1;
        id
    }

    fn matches_expression(
        &mut self,
        first_char: char,
        match_char: &char,
        left: ExpressionOperatorTokenType,
        right: ExpressionOperatorTokenType,
    ) -> MatchedExpression {
        if let Some(next) = self.source_iter.peek() {
            return if next == match_char {
                self.source_iter.next();
                MatchedExpression {
                    token_type: left,
                    lexeme: vec![first_char, *match_char],
                }
            } else {
                MatchedExpression {
                    token_type: right,
                    lexeme: vec![first_char],
                }
            };
        }
        MatchedExpression {
            token_type: right,
            lexeme: vec![first_char],
        }
    }

    fn scan_slash(&mut self) -> Option<TokenType> {
        let mut is_advanced = false;
        while let Some(next) = self.source_iter.peek() {
            match next {
                '/' => is_advanced = true,
                '\n' => return None,
                _ => {
                    if !is_advanced {
                        return Some(TokenType::SingleChar(SingleCharTokenType::Slash));
                    }
                }
            }
            self.source_iter.next();
        }
        None
    }

    fn scan_string_literal(&mut self) -> Option<(String, u32)> {
        let mut result = String::new();
        let mut new_line_count = 0u32;
        for next in &mut self.source_iter {
            match next {
                '\n' => new_line_count += 1,
                '"' => return Some((result, new_line_count)),
                _ => result.push(next),
            }
        }
        None
    }

    fn scan_number(&mut self, first_char: char) -> (f64, String) {
        let mut result = self.scan_digits();
        result.insert(0, first_char);

        if let Some(&'.') = self.source_iter.peek() {
            let _ = self.source_iter.advance_cursor();
            if let Some(character) = self.source_iter.peek() {
                if character.is_digit(10) {
                    self.source_iter.reset_cursor();
                    self.source_iter.next();
                    let digits = self.scan_digits();
                    result.push('.');
                    result.extend(digits);
                    return Scanner::chars_to_number(&result);
                }
            }
        }
        self.source_iter.reset_cursor();
        Scanner::chars_to_number(&result)
    }

    fn scan_digits(&mut self) -> Vec<char> {
        let mut result = Vec::new();
        while let Some(next) = self.source_iter.peek() {
            if next.is_digit(10) {
                result.push(*next);
                self.source_iter.next();
            } else {
                break;
            }
        }
        result
    }

    fn chars_to_number(chars: &[char]) -> (f64, String) {
        let string: String = chars.iter().collect();
        (string.parse().unwrap(), string)
    }

    fn scan_identifier(
        &mut self,
        first_char: char,
        keywords: &HashMap<String, KeywordTokenType>,
    ) -> (TokenType, String) {
        let mut keyword: Vec<char> = vec![first_char];

        while let Some(next) = self.source_iter.peek() {
            if next.is_alphanumeric() {
                keyword.push(*next);
                self.source_iter.next();
            } else {
                break;
            }
        }
        let string: String = keyword.iter().collect();
        if let Some(keyword) = keywords.get(&string) {
            (TokenType::Keyword(keyword.clone()), string)
        } else {
            let lexeme = string.clone();
            (
                TokenType::Literal(LiteralTokenType::Identifier(string)),
                lexeme,
            )
        }
    }
}
