use crate::token::Token;
use crate::token_type::{
    Delimeter, ExpressionOperatorTokenType, KeywordTokenType, LiteralTokenType,
    SingleCharTokenType, TokenType,
};
use peekmore::{PeekMore, PeekMoreIterator};
use std::collections::HashMap;
use std::str::Chars;

pub struct Scanner {
    source: String,
    keywords: HashMap<String, KeywordTokenType>,
}

struct MatchedExpression {
    token_type: ExpressionOperatorTokenType,
    lexeme: Vec<char>,
}

impl MatchedExpression {
    fn make_token(self, line: u32) -> Token {
        Token::new(
            TokenType::ExpressionOperator(self.token_type),
            self.lexeme,
            line,
        )
    }
}

enum CharacterScanResult {
    Token(Token),
    StringLiteral(Token, u32),
    NewLine,
    Skipped,
    Err(String),
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            source,
            keywords: KeywordTokenType::make_keywords(),
        }
    }

    pub fn scan_tokens(self) -> Vec<Token> {
        let mut tokens = vec![];
        let mut source_iter = self.source.chars().peekmore();
        let mut line = 1u32;

        while let Some(character) = source_iter.next() {
            match Scanner::scan_token(character, line, &mut source_iter, &self.keywords) {
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
                CharacterScanResult::Err(message) => {}
                CharacterScanResult::Skipped => {}
            }
        }
        tokens.push(Token::new(TokenType::EOF, Vec::new(), line));
        tokens
    }

    fn scan_token(
        character: char,
        line: u32,
        source_iter: &mut PeekMoreIterator<Chars>,
        keywords: &HashMap<String, KeywordTokenType>,
    ) -> CharacterScanResult {
        let make_result = |token| CharacterScanResult::Token(token);
        let make_token =
            |token_type| make_result(Token::new_single_char(token_type, character, line));

        match character {
            '(' => make_token(TokenType::OpenDelimeter(Delimeter::Paren)),
            ')' => make_token(TokenType::CloseDelimeter(Delimeter::Paren)),
            '{' => make_token(TokenType::OpenDelimeter(Delimeter::Brace)),
            '}' => make_token(TokenType::CloseDelimeter(Delimeter::Brace)),
            '[' => make_token(TokenType::OpenDelimeter(Delimeter::Bracket)),
            ']' => make_token(TokenType::CloseDelimeter(Delimeter::Bracket)),
            ',' => make_token(TokenType::SingleChar(SingleCharTokenType::Comma)),
            '.' => make_token(TokenType::SingleChar(SingleCharTokenType::Dot)),
            '-' => make_token(TokenType::SingleChar(SingleCharTokenType::Minus)),
            '+' => make_token(TokenType::SingleChar(SingleCharTokenType::Plus)),
            ';' => make_token(TokenType::SingleChar(SingleCharTokenType::Simicolon)),
            '*' => make_token(TokenType::SingleChar(SingleCharTokenType::Star)),
            '!' => make_result(
                Scanner::matches_expression(
                    '!',
                    &'=',
                    source_iter,
                    ExpressionOperatorTokenType::NotEqual,
                    ExpressionOperatorTokenType::Not,
                )
                .make_token(line),
            ),
            '=' => make_result(
                Scanner::matches_expression(
                    '=',
                    &'=',
                    source_iter,
                    ExpressionOperatorTokenType::EqualEqual,
                    ExpressionOperatorTokenType::Equal,
                )
                .make_token(line),
            ),
            '>' => make_result(
                Scanner::matches_expression(
                    '>',
                    &'=',
                    source_iter,
                    ExpressionOperatorTokenType::GreaterEqual,
                    ExpressionOperatorTokenType::Greater,
                )
                .make_token(line),
            ),
            '<' => make_result(
                Scanner::matches_expression(
                    '<',
                    &'=',
                    source_iter,
                    ExpressionOperatorTokenType::LessEqual,
                    ExpressionOperatorTokenType::Less,
                )
                .make_token(line),
            ),
            '/' => {
                if let Some(token_type) = Scanner::scan_slash(source_iter) {
                    make_token(token_type)
                } else {
                    CharacterScanResult::NewLine
                }
            }
            ' ' | '\r' | '\t' => CharacterScanResult::Skipped,
            '"' => Scanner::scan_string_literal(source_iter).map_or(
                CharacterScanResult::Err(format!("Unterminated string")),
                |(literal, line_count)| {
                    let lexeme = literal.chars().clone().collect();
                    let token_type = TokenType::Literal(LiteralTokenType::String(literal));
                    let token = Token::new(token_type, lexeme, line);
                    return CharacterScanResult::StringLiteral(token, line_count);
                },
            ),
            '\n' => CharacterScanResult::NewLine,
            _ => {
                if character.is_digit(10) {
                    let (number, lexeme) = Scanner::scan_number(character, source_iter);
                    let token_type = TokenType::Literal(LiteralTokenType::Number(number));
                    let token = Token::new(token_type, lexeme.chars().collect(), line);
                    make_result(token)
                } else if character.is_alphanumeric() {
                    let (token_type, lexeme) =
                        Scanner::scan_identifier(character, source_iter, keywords);
                    let token = Token::new(token_type, lexeme.chars().collect(), line);
                    make_result(token)
                } else {
                    CharacterScanResult::Err(format!("Unknown symbol"))
                }
            }
        }
    }

    fn matches_expression(
        first_char: char,
        match_char: &char,
        source_iter: &mut PeekMoreIterator<Chars>,
        left: ExpressionOperatorTokenType,
        right: ExpressionOperatorTokenType,
    ) -> MatchedExpression {
        if let Some(next) = source_iter.peek() {
            if next == match_char {
                source_iter.next();
                return MatchedExpression {
                    token_type: left,
                    lexeme: vec![first_char, *match_char],
                };
            } else {
                return MatchedExpression {
                    token_type: right,
                    lexeme: vec![first_char],
                };
            }
        }
        MatchedExpression {
            token_type: right,
            lexeme: vec![first_char],
        }
    }

    fn scan_slash(source_iter: &mut PeekMoreIterator<Chars>) -> Option<TokenType> {
        let mut is_advanced = false;
        while let Some(next) = source_iter.peek() {
            match next {
                '/' if !is_advanced => {
                    return Some(TokenType::SingleChar(SingleCharTokenType::Slash))
                }
                '\n' => return None,
                _ => {
                    is_advanced = true;
                    source_iter.next();
                }
            }
        }
        None
    }

    fn scan_string_literal(source_iter: &mut PeekMoreIterator<Chars>) -> Option<(String, u32)> {
        let mut result = String::new();
        let mut new_line_count = 0u32;
        while let Some(next) = source_iter.next() {
            match next {
                '\n' => new_line_count += 1,
                '"' => return Some((result, new_line_count)),
                _ => result.push(next),
            }
        }
        None
    }

    fn scan_number(first_char: char, source_iter: &mut PeekMoreIterator<Chars>) -> (f32, String) {
        let mut result = Scanner::scan_digits(source_iter);
        result.insert(0, first_char);

        if let Some(&'.') = source_iter.peek() {
            let _ = source_iter.advance_cursor();
            if let Some(character) = source_iter.peek() {
                if character.is_digit(10) {
                    source_iter.reset_cursor();
                    source_iter.next();
                    let digits = Scanner::scan_digits(source_iter);
                    result.push('.');
                    result.extend(digits);
                    return Scanner::chars_to_number(&result);
                }
            }
        }
        source_iter.reset_cursor();
        Scanner::chars_to_number(&result)
    }

    fn scan_digits(source_iter: &mut PeekMoreIterator<Chars>) -> Vec<char> {
        let mut result = Vec::new();
        while let Some(next) = source_iter.peek() {
            if next.is_digit(10) {
                result.push(*next);
                source_iter.next();
            } else {
                break;
            }
        }
        result
    }

    fn chars_to_number(chars: &Vec<char>) -> (f32, String) {
        let string: String = chars.iter().collect();
        (string.parse().unwrap(), string)
    }

    fn scan_identifier(
        first_char: char,
        source_iter: &mut PeekMoreIterator<Chars>,
        keywords: &HashMap<String, KeywordTokenType>,
    ) -> (TokenType, String) {
        let mut keyword: Vec<char> = vec![first_char];

        while let Some(next) = source_iter.peek() {
            if next.is_alphanumeric() {
                keyword.push(*next);
                source_iter.next();
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
