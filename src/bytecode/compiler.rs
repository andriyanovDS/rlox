use crate::bytecode::compiler::CompileError::TokenError;
use crate::bytecode::token::Lexeme;
use super::chunk::Chunk;
use super::op_code::OpCode;
use super::scanner::ScanError;
use super::token::{Token, TokenType};
use super::value::Value;
use super::scanner::Scanner;
use super::parse_rule::{ParseType, Precedence, ParseRule};

pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    source: &'a str,
    chunk: Chunk,
    parse_rules: [ParseRule<'a>; 39],
    previous_token: Option<Token>,
    current_token: Option<Token>,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            scanner: Scanner::new(&source),
            source,
            chunk: Chunk::new(),
            parse_rules: Compiler::make_parse_rules(),
            previous_token: None,
            current_token: None,
        }
    }

    pub fn compile(&mut self) {
        if let Err(error) = self.start_compile() {
            self.handle_error(&error);
        }
    }

    fn start_compile(&mut self) -> CompileResult {
        self.advance()?;
        self.expression()?;
        self.consume(TokenType::Eof, "Expect end of expression.")
    }

    fn end_compiler(&mut self, line: usize) {
        self.chunk.push_code(OpCode::Return, line);
    }

    fn advance(&mut self) -> CompileResult {
        self.previous_token = self.current_token.take();
        let token = self.scanner.scan_token().map_err(CompileError::ScanError)?;
        self.current_token = Some(token);
        Ok(())
    }

    fn consume(&mut self, expected_type: TokenType, error_message: &'static str) -> CompileResult {
        let current_token = self.current_token();
        if expected_type == current_token.token_type {
            self.advance()
        } else {
            Err(CompileError::make_from_token(current_token, error_message))
        }
    }

    fn expression(&mut self) -> CompileResult {
        self.parse_precedence(Precedence::Assignment)
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> CompileResult {
        self.advance()?;
        let token = self.previous_token();
        let rule = self.parse_rule(&token.token_type);
        match rule.parse_type.prefix() {
            Some(func) => func(self),
            None => Err(CompileError::make_from_token(token, "Expect expression."))
        }?;
        let precedence_int: u8 = precedence as u8;
        loop {
            let current_rule = self.parse_rule(&self.current_token().token_type);
            if (current_rule.precedence.clone() as u8) < precedence_int {
                break Ok(());
            }
            self.advance()?;
            let token = self.previous_token();
            let rule = self.parse_rule(&token.token_type);
            match rule.parse_type.infix() {
                Some(func) => func(self),
                None => Err(CompileError::make_from_token(token, "Expect expression."))
            }?;
        }
    }

    fn grouping(&mut self) -> CompileResult {
        self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after expression.")
    }

    fn unary(&mut self) -> CompileResult {
        let previous_token = self.previous_token();
        let line = previous_token.line;
        let token_type = previous_token.token_type;
        self.parse_precedence(Precedence::Unary)?;
        match token_type {
            TokenType::Minus => {
                self.chunk.push_code(OpCode::Negate, line)
            },
            _ => {}
        }
        Ok(())
    }

    fn binary(&mut self) -> CompileResult {
        let previous_token = self.previous_token();
        let token_type = previous_token.token_type.clone();
        let token_line = previous_token.line;
        let rule = self.parse_rule(&token_type);
        let precedence = Precedence::try_from((rule.precedence as u8) + 1).unwrap();
        self.parse_precedence(precedence)?;
        let op_code = match token_type {
            TokenType::Plus => Some(OpCode::Add),
            TokenType::Minus => Some(OpCode::Subtract),
            TokenType::Star => Some(OpCode::Multiply),
            TokenType::Slash => Some(OpCode::Divide),
            _ => None
        };
        if let Some(op_code) = op_code {
            self.chunk.push_code(op_code, token_line);
        }
        Ok(())
    }

    fn emit_number(&mut self) -> CompileResult {
        let number: f32 = self.previous_token().lexeme
            .as_ref()
            .expect("Only EOF token can not have a lexeme")
            .make_slice(self.source)
            .parse()
            .expect("Invalid number parsed");
        let index = self.chunk.add_constant(Value::Double(number));
        self.chunk.push_constant(index, self.previous_token().line);
        Ok(())
    }

    fn parse_rule(&self, token_type: &TokenType) -> &ParseRule<'a> {
        &self.parse_rules[*token_type as usize]
    }

    fn current_token(&self) -> &Token {
        self.current_token.as_ref().unwrap()
    }

    fn previous_token(&self) -> &Token {
        self.previous_token.as_ref().unwrap()
    }

    fn handle_error(&self, error: &CompileError) {
        match error {
            CompileError::ScanError(error) => {
                eprintln!("[line {}] Error: {}", error.line, error.message);
            }
            CompileError::TokenError { line, lexeme, message } => {
                if let Some(lexeme) = lexeme.as_ref() {
                    eprintln!("[line {}] Error at {:?}: {}", line, lexeme.make_slice(self.source), message);
                } else {
                    eprintln!("[line {}] Error at end: {}", line, message);
                }
            }
        }
    }

    fn make_parse_rules<'c>() -> [ParseRule<'c>; 39] {
        return [
            ParseRule {
                parse_type: ParseType::Prefix(Compiler::grouping),
                precedence: Precedence::None
            },                                                                       // TokenType::LeftParen
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::RightParen
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::LeftBrace
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::RightBrace
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Comma
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Dot
            ParseRule {
                parse_type: ParseType::Both {
                    prefix: Compiler::unary,
                    infix: Compiler::binary,
                },
                precedence: Precedence::Term
            },                                                                       // TokenType::Minus
            ParseRule {
                parse_type: ParseType::Infix(Compiler::binary),
                precedence: Precedence::Term
            }, // TokenType::Plus
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Semicolon
            ParseRule {
                parse_type: ParseType::Infix(Compiler::binary),
                precedence: Precedence::Factor
            },                                                                       // TokenType::Slash
            ParseRule {
                parse_type: ParseType::Infix(Compiler::binary),
                precedence: Precedence::Factor
            },                                                                       // TokenType::Star
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Bang
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::BangEqual
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Equal
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::EqualEqual
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Greater
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::GreaterEqual
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Less
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::LessEqual
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Identifier
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::String
            ParseRule {
                parse_type: ParseType::Prefix(Compiler::emit_number),
                precedence: Precedence::None
            },                                                                       // TokenType::Number
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::And
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Class
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Else
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::False
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::For
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Fun
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::If
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Nil
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Or
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Print
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Return
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Super
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::This
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::True
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Var
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::While
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Eof
        ];
    }
}

pub enum CompileError {
    ScanError(ScanError),
    TokenError {
        line: usize,
        lexeme: Option<Lexeme>,
        message: &'static str,
    }
}

impl CompileError {
    fn make_from_token(token: &Token, message: &'static str) -> Self {
        CompileError::TokenError {
            line: token.line,
            lexeme: token.lexeme,
            message
        }
    }
}

pub type CompileResult = Result<(), CompileError>;
