use std::collections::HashMap;

#[derive(Debug)]
pub enum TokenType {
    SingleChar(SingleCharTokenType),
    CloseDelimeter(Delimeter),
    OpenDelimeter(Delimeter),
    ExpressionOperator(ExpressionOperatorTokenType),
    Literal(LiteralTokenType),
    Keyword(KeywordTokenType),
    EOF,
}

#[derive(Debug)]
pub enum Delimeter {
    Paren,
    Bracket,
    Brace,
}

#[derive(Debug)]
pub enum SingleCharTokenType {
    Comma,
    Dot,
    Minus,
    Plus,
    Simicolon,
    Slash,
    Star,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum LiteralTokenType {
    Identifier(String),
    String(String),
    Number(f32),
}

#[derive(Debug, Clone)]
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
