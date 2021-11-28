use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    SingleChar(SingleCharTokenType),
    CloseDelimiter(Delimiter),
    OpenDelimiter(Delimiter),
    ExpressionOperator(ExpressionOperatorTokenType),
    Literal(LiteralTokenType),
    Keyword(KeywordTokenType),
    Eof,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Delimiter {
    Paren,
    Bracket,
    Brace,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SingleCharTokenType {
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionOperatorTokenType {
    Not,
    NotEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralTokenType {
    Identifier(String),
    String(String),
    Number(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeywordTokenType {
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
}

impl KeywordTokenType {
    pub fn make_keywords() -> HashMap<String, KeywordTokenType> {
        vec![
            ("and", KeywordTokenType::And),
            ("class", KeywordTokenType::Class),
            ("else", KeywordTokenType::Else),
            ("false", KeywordTokenType::False),
            ("for", KeywordTokenType::For),
            ("fun", KeywordTokenType::Fun),
            ("if", KeywordTokenType::If),
            ("nil", KeywordTokenType::Nil),
            ("or", KeywordTokenType::Or),
            ("print", KeywordTokenType::Print),
            ("return", KeywordTokenType::Return),
            ("super", KeywordTokenType::Super),
            ("this", KeywordTokenType::This),
            ("true", KeywordTokenType::True),
            ("var", KeywordTokenType::Var),
            ("while", KeywordTokenType::While),
        ]
        .into_iter()
        .map(|(key, value)| (String::from(key), value))
        .collect()
    }
}
