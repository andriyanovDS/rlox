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

    pub fn compile(&mut self) -> Result<(), ScanError> {
        self.advance()?;
        self.expression()?;
        self.consume(TokenType::Eof, "Expect end of expression.")
    }

    fn end_compiler(&mut self, line: usize) {
        self.chunk.push_code(OpCode::Return, line);
    }

    fn advance(&mut self) -> Result<(), ScanError> {
        self.previous_token = self.current_token.take();
        let token = self.scanner.scan_token()?;
        self.current_token = Some(token);
        Ok(())
    }

    fn consume(&mut self, expected_type: TokenType, error_message: &'static str) -> Result<(), ScanError> {
        let current_token = self.current_token();
        if expected_type == current_token.token_type {
            self.advance()
        } else {
            Err(ScanError { line: current_token.line, message: error_message })
        }
    }

    fn expression(&mut self) -> Result<(), ScanError> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<(), ScanError> {
        self.advance()?;
        let token = self.previous_token();
        let rule = self.parse_rule(&token.token_type);
        match rule.parse_type.prefix() {
            Some(func) => func(self),
            None => Err(ScanError { line: token.line, message: "Expect expression." })
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
                None => Err(ScanError { line: token.line, message: "Expect expression." })
            }?;
        }
    }

    fn grouping(&mut self) -> Result<(), ScanError> {
        self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after expression.")
    }

    fn unary(&mut self) -> Result<(), ScanError> {
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

    fn binary(&mut self) -> Result<(), ScanError> {
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

    fn emit_number(&mut self) -> Result<(), ScanError> {
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

    fn print_error(message: &'static str, line: usize) {
        eprintln!("[line {}] Error: {}", line, message);
    }

    fn print_token_error(&self, token: &Token, message: &'static str) {
        if token.token_type == TokenType::Eof {
            eprintln!("[line {}] Error at end: {}", token.line, message);
        } else {
            let lexeme = token.lexeme.as_ref().unwrap().make_slice(self.source);
            eprintln!("[line {}] Error at {:?}: {}", token.line, lexeme, message);
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
