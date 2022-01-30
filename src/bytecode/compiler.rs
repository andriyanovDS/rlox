use std::cell::RefCell;
use std::rc::Rc;
use super::hash_table::HashTable;
use super::object_string::ObjectString;
use super::token::Lexeme;
use super::chunk::Chunk;
use super::op_code::OpCode;
use super::scanner::ScanError;
use super::token::{Token, TokenType};
use super::value::Value;
use super::scanner::Scanner;
use super::parse_rule::{ParseType, Precedence, ParseRule};

pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    interned_strings: Rc<RefCell<HashTable<Rc<ObjectString>, ()>>>,
    source: &'a str,
    chunk: Chunk,
    parse_rules: [ParseRule<'a>; 39],
    previous_token: Option<Token>,
    current_token: Option<Token>,
}

impl<'a> Compiler<'a> {
    pub fn new(
        source: &'a str,
        interned_strings: Rc<RefCell<HashTable<Rc<ObjectString>, ()>>>
    ) -> Self {
        Self {
            scanner: Scanner::new(source),
            interned_strings,
            source,
            chunk: Chunk::new(),
            parse_rules: Compiler::make_parse_rules(),
            previous_token: None,
            current_token: None,
        }
    }

    pub fn chunk(&self) -> &Chunk { &self.chunk }

    pub fn compile(&mut self) {
        if let Err(error) = self.start_compilation() {
            self.handle_error(&error);
        }
    }

    fn start_compilation(&mut self) -> CompilationResult {
        self.advance()?;
        while self.current_token().token_type != TokenType::Eof {
            if let Err(error) = self.declaration() {
                self.handle_error(&error);
                self.synchronize();
            }
        }
        let line = self.previous_token().line;
        self.consume(TokenType::Eof, "Expect end of expression.")?;
        self.end_compiler(line);
        Ok(())
    }

    fn end_compiler(&mut self, line: usize) {
        self.chunk.push_code(OpCode::Return, line);
    }

    fn advance(&mut self) -> CompilationResult {
        self.previous_token = self.current_token.take();
        let token = self.scanner.scan_token().map_err(CompileError::ScanError)?;
        self.current_token = Some(token);
        Ok(())
    }

    fn consume(
        &mut self,
        expected_type: TokenType,
        error_message: &'static str
    ) -> CompilationResult {
        let current_token = self.current_token();
        if expected_type == current_token.token_type {
            self.advance()
        } else {
            Err(CompileError::make_from_token(current_token, error_message))
        }
    }

    #[inline]
    fn declaration(&mut self) -> CompilationResult {
        if self.current_token().token_type == TokenType::Var {
            self.advance()?;
            self.variable_declaration()
        } else {
            self.statement()
        }
    }

    fn statement(&mut self) -> CompilationResult {
        let current_token_type = self.current_token().token_type;
        self.advance()?;
        match current_token_type {
            TokenType::Print => self.print_statement(),
            _ => self.expression_statement()
        }
    }

    fn variable_declaration(&mut self) -> CompilationResult {
        let global = self.parse_variable("Expect variable name.")?;
        let line = self.previous_token().line;
        if self.current_token().token_type == TokenType::Equal {
            self.advance()?;
            self.expression()?;
        } else {
            self.chunk.push_code(OpCode::Nil, line);
        }
        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration.")?;
        self.chunk.define_global_variable(global, line);
        Ok(())
    }

    #[inline]
    fn parse_variable(&mut self, error_message: &'static str) -> Result<Rc<ObjectString>, CompileError> {
        if self.current_token().token_type == TokenType::Identifier {
            self.advance()?;
            Ok(self.intern_string())
        } else {
            Err(CompileError::make_from_token(self.current_token(), error_message))
        }
    }

    #[inline]
    fn print_statement(&mut self) -> CompilationResult {
        self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        self.push_code(OpCode::Print);
        Ok(())
    }

    #[inline]
    fn expression_statement(&mut self) -> CompilationResult {
        self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        self.push_code(OpCode::Pop);
        Ok(())
    }

    #[inline]
    fn expression(&mut self) -> CompilationResult {
        self.parse_precedence(Precedence::Assignment)
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> CompilationResult {
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
            if (current_rule.precedence as u8) < precedence_int {
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

    #[inline]
    fn grouping(&mut self) -> CompilationResult {
        self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after expression.")
    }

    fn unary(&mut self) -> CompilationResult {
        let previous_token = self.previous_token();
        let line = previous_token.line;
        let token_type = previous_token.token_type;
        self.parse_precedence(Precedence::Unary)?;
        match token_type {
            TokenType::Minus => self.chunk.push_code(OpCode::Negate, line),
            TokenType::Bang => self.chunk.push_code(OpCode::Not, line),
            _ => {}
        }
        Ok(())
    }

    fn binary(&mut self) -> CompilationResult {
        let previous_token = self.previous_token();
        let token_type = previous_token.token_type;
        let token_line = previous_token.line;
        let rule = self.parse_rule(&token_type);
        let precedence = Precedence::try_from((rule.precedence as u8) + 1).unwrap();
        self.parse_precedence(precedence)?;
        match token_type {
            TokenType::Plus => self.chunk.push_code(OpCode::Add, token_line),
            TokenType::Minus => self.chunk.push_code(OpCode::Subtract, token_line),
            TokenType::Star => self.chunk.push_code(OpCode::Multiply, token_line),
            TokenType::Slash => self.chunk.push_code(OpCode::Divide, token_line),
            TokenType::BangEqual => {
                self.chunk.push_code(OpCode::Equal, token_line);
                self.chunk.push_code(OpCode::Not, token_line);
            }
            TokenType::EqualEqual => self.chunk.push_code(OpCode::Equal, token_line),
            TokenType::Greater => self.chunk.push_code(OpCode::Greater, token_line),
            TokenType::GreaterEqual => {
                self.chunk.push_code(OpCode::Less, token_line);
                self.chunk.push_code(OpCode::Not, token_line);
            }
            TokenType::Less => self.chunk.push_code(OpCode::Less, token_line),
            TokenType::LessEqual => {
                self.chunk.push_code(OpCode::Greater, token_line);
                self.chunk.push_code(OpCode::Not, token_line);
            }
            _ => {}
        };
        Ok(())
    }

    fn emit_number(&mut self) -> CompilationResult {
        let number: f32 = self.previous_token().lexeme
            .as_ref()
            .expect("Only EOF token can not have a lexeme")
            .make_slice(self.source)
            .parse()
            .expect("Invalid number parsed");
        self.chunk.add_constant(Value::Number(number), self.previous_token().line);
        Ok(())
    }

    fn literal(&mut self) -> CompilationResult {
        let previous_token = self.previous_token();
        let line = previous_token.line;
        match previous_token.token_type {
            TokenType::True => self.chunk.push_code(OpCode::True, line),
            TokenType::False => self.chunk.push_code(OpCode::False, line),
            TokenType::Nil => self.chunk.push_code(OpCode::Nil, line),
            _ => {}
        }
        Ok(())
    }

    fn string(&mut self) -> CompilationResult {
        let object = self.intern_string();
        self.chunk.add_constant(Value::String(object), self.previous_token().line);
        Ok(())
    }

    fn variable(&mut self) -> CompilationResult {
        let object = self.intern_string();
        self.chunk.get_global_variable(object, self.previous_token().line);
        Ok(())
    }

    #[inline]
    fn intern_string(&mut self) -> Rc<ObjectString> {
        let token = self.previous_token();
        let lexeme = token.lexeme
            .as_ref()
            .unwrap()
            .make_slice(self.source)
            .to_string();
        let mut strings = self.interned_strings.as_ref().borrow_mut();
        strings.find_string_or_insert_new(lexeme)
    }

    #[inline]
    fn parse_rule(&self, token_type: &TokenType) -> &ParseRule<'a> {
        &self.parse_rules[*token_type as usize]
    }

    #[inline]
    fn push_code(&mut self, code: OpCode) {
        self.chunk.push_code(code, self.current_token().line);
    }

    fn synchronize(&mut self) {
        loop {
            let current_token_type = self.current_token().token_type;
            match current_token_type {
                TokenType::Eof | TokenType::Class | TokenType::For
                | TokenType::Fun | TokenType::Var | TokenType::If
                | TokenType::Print | TokenType::Return => {
                    return;
                }
                _ if self.previous_token().token_type == TokenType::Semicolon => {
                    return;
                }
                _ => {
                    let _ = self.advance();
                }
            }
        }
    }

    #[inline]
    fn current_token(&self) -> &Token {
        self.current_token.as_ref().unwrap()
    }

    #[inline]
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
            ParseRule {
                parse_type: ParseType::Prefix(Compiler::unary),
                precedence: Precedence::None
            }, // TokenType::Bang
            ParseRule {
                parse_type: ParseType::Infix(Compiler::binary),
                precedence: Precedence::Equality
            }, // TokenType::BangEqual
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Equal
            ParseRule {
                parse_type: ParseType::Infix(Compiler::binary),
                precedence: Precedence::Equality
            }, // TokenType::EqualEqual
            ParseRule {
                parse_type: ParseType::Infix(Compiler::binary),
                precedence: Precedence:: Comparison
            }, // TokenType::Greater
            ParseRule {
                parse_type: ParseType::Infix(Compiler::binary),
                precedence: Precedence:: Comparison
            }, // TokenType::GreaterEqual
            ParseRule {
                parse_type: ParseType::Infix(Compiler::binary),
                precedence: Precedence:: Comparison
            }, // TokenType::Less
            ParseRule {
                parse_type: ParseType::Infix(Compiler::binary),
                precedence: Precedence:: Comparison
            }, // TokenType::LessEqual
            ParseRule {
                parse_type: ParseType::Prefix(Compiler::variable),
                precedence: Precedence::None
            }, // TokenType::Identifier
            ParseRule {
                parse_type: ParseType::Prefix(Compiler::string),
                precedence: Precedence::None
            }, // TokenType::String
            ParseRule {
                parse_type: ParseType::Prefix(Compiler::emit_number),
                precedence: Precedence::None
            },                                                                       // TokenType::Number
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::And
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Class
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Else
            ParseRule {
                parse_type: ParseType::Prefix(Compiler::literal),
                precedence: Precedence::None
            }, // TokenType::False
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::For
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Fun
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::If
            ParseRule {
                parse_type: ParseType::Prefix(Compiler::literal),
                precedence: Precedence::None
            }, // TokenType::Nil
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Or
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Print
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Return
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Super
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::This
            ParseRule {
                parse_type: ParseType::Prefix(Compiler::literal),
                precedence: Precedence::None
            }, // TokenType::True
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

pub type CompilationResult = Result<(), CompileError>;
