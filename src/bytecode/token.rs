#[derive(Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: Option<Lexeme>,
    pub line: usize,
}

#[derive(Copy, Clone)]
pub struct Lexeme {
    pub start: usize,
    pub length: usize
}

impl Lexeme {
    pub fn make_slice<'a>(&self, string: &'a str) -> &'a str {
        &string[self.start..self.start + self.length]
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    Identifier,
    String,
    Number,
    // Keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fun,
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
    Continue,
    Eof
}
