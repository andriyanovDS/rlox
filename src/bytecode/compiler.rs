use std::cell::{Ref, RefCell, RefMut};
use std::mem;
use std::rc::Rc;
use super::scope::Scope;
use super::hash_table::HashTable;
use super::token::Lexeme;
use super::chunk::Chunk;
use super::op_code::OpCode;
use super::scanner::ScanError;
use super::token::{Token, TokenType};
use super::value::{Value, object_function::ObjectFunction, object_string};
use object_string::{ObjectString, INIT_KEYWORD};
use super::scanner::Scanner;
use super::parse_rule::{ParseType, Precedence, ParseRule};

pub struct Compiler<'a> {
    scanner: Rc<RefCell<Scanner<'a>>>,
    interned_strings: Rc<RefCell<HashTable<Rc<ObjectString>, ()>>>,
    string_constants: HashTable<Rc<ObjectString>, u8>,
    scope: Rc<RefCell<Scope>>,
    source: &'a str,
    chunk: Chunk,
    parse_rules: &'a [ParseRule<'a>; 39],
    previous_token: Option<Token>,
    current_token: Option<Token>,
    loop_context: Option<LoopContext>,
    is_inside_class: bool,
}

pub struct CompilerContext<'a> {
    scanner: Rc<RefCell<Scanner<'a>>>,
    source: &'a str,
    parse_rules: &'a [ParseRule<'a>; 39],
    interned_strings: Rc<RefCell<HashTable<Rc<ObjectString>, ()>>>,
    previous_token: Option<Token>,
    current_token: Option<Token>,
    enclosing_scope: Rc<RefCell<Scope>>,
    is_inside_class: bool,
}

impl<'a> CompilerContext<'a>  {
    pub fn new(
        source: &'a str,
        parse_rules: &'a [ParseRule<'a>; 39],
        interned_strings: Rc<RefCell<HashTable<Rc<ObjectString>, ()>>>,
        is_inside_class: bool,
    ) -> Self {
        Self {
            scanner: Rc::new(RefCell::new(Scanner::new(source))),
            source,
            parse_rules,
            interned_strings,
            previous_token: None,
            current_token: None,
            enclosing_scope: Rc::new(RefCell::new(Scope::new(None))),
            is_inside_class,
        }
    }
}

impl<'a> Compiler<'a> {
    pub fn new(context: CompilerContext<'a>) -> Self {
        Self {
            scanner: context.scanner,
            interned_strings: context.interned_strings,
            string_constants: HashTable::new(),
            scope: Rc::new(RefCell::new(Scope::new(Some(context.enclosing_scope)))),
            source: context.source,
            chunk: Chunk::new(),
            parse_rules: context.parse_rules,
            previous_token: context.previous_token,
            current_token: context.current_token,
            loop_context: None,
            is_inside_class: context.is_inside_class,
        }
    }

    pub fn compile(&mut self) -> Option<&Chunk> {
        if let Err(error) = self.start_compilation() {
            self.handle_error(&error);
            None
        } else {
            Some(&self.chunk)
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
        self.emit_return(line, &FunctionType::Function);
        Ok(())
    }

    #[inline]
    fn modify_chunk<F, R>(&mut self, callback: F) -> R where F: FnOnce(&mut Chunk) -> R {
        callback(&mut self.chunk)
    }

    #[inline]
    fn current_chunk_size(&self) -> usize {
        self.chunk.codes.length
    }

    #[inline]
    fn scope(&self) -> Ref<Scope> {
        self.scope.as_ref().borrow()
    }

    #[inline]
    fn scope_mut(&self) -> RefMut<Scope> {
        self.scope.as_ref().borrow_mut()
    }

    fn advance(&mut self) -> CompilationResult {
        self.previous_token = self.current_token.take();
        let token = self.scanner
            .as_ref()
            .borrow_mut()
            .scan_token()
            .map_err(CompileError::ScanError)?;
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
        let token_type = self.current_token().token_type;
        match token_type {
            TokenType::Var => {
                self.advance()?;
                self.variable_declaration()
            }
            TokenType::Fun => {
                self.advance()?;
                self.function_declaration()
            }
            TokenType::Class => {
                self.advance()?;
                self.class_declaration()
            }
            _ => self.statement()
        }
    }

    fn statement(&mut self) -> CompilationResult {
        let current_token_type = self.current_token().token_type;
        match current_token_type {
            TokenType::Print => {
                self.advance()?;
                self.print_statement()
            }
            TokenType::If => {
                self.advance()?;
                self.if_statement()
            }
            TokenType::Return => {
                self.advance()?;
                self.return_statement()
            }
            TokenType::While => {
                self.advance()?;
                self.while_statement()
            }
            TokenType::For => {
                self.advance()?;
                self.for_statement()
            }
            TokenType::Continue => {
                self.advance()?;
                self.continue_statement()
            }
            TokenType::LeftBrace => self.parse_block(),
            _ => self.expression_statement()
        }
    }

    fn return_statement(&mut self) -> CompilationResult {
        if self.current_token().token_type == TokenType::Semicolon {
            self.emit_return(self.current_token().line, &FunctionType::Function);
            self.advance()
        } else {
            self.expression()?;
            self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
            self.chunk.push_code(OpCode::Return, self.previous_token().line);
            Ok(())
        }
    }

    fn variable_declaration(&mut self) -> CompilationResult {
        let index = self.parse_variable("Expect variable name.")?;
        let line = self.previous_token().line;
        if self.current_token().token_type == TokenType::Equal {
            self.advance()?;
            self.expression()?;
        } else {
            self.modify_chunk(|chunk| chunk.push_code(OpCode::Nil, line));
        }
        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration.")?;
        self.define_variable(index, line);
        Ok(())
    }

    fn declare_local_variable(&mut self) -> CompilationResult {
        let mut scope = self.scope_mut();
        assert!(!scope.is_global_scope());
        let token = self.previous_token().clone();
        if scope.is_redeclaration(&token, self.source) {
            Err(CompileError::make_from_token(&token, "Already a variable with this name in this scope."))
        } else {
            scope.add_local(token)
        }
    }

    #[inline]
    fn parse_variable(&mut self, error_message: &'static str) -> Result<Option<usize>, CompileError> {
        if self.current_token().token_type != TokenType::Identifier {
            return Err(CompileError::make_from_token(self.current_token(), error_message));
        }
        self.advance()?;
        if self.scope().is_global_scope() {
            let object = self.intern_string();
            let index = self.modify_chunk(|chunk| {
                chunk.push_constant_to_pool(Value::String(object))
            });
            Ok(Some(index))
        } else {
            self.declare_local_variable()?;
            Ok(None)
        }
    }

    #[inline]
    fn define_variable(&mut self, global_index: Option<usize>, line: usize) {
        match global_index {
            Some(index) => {
                self.modify_chunk(|chunk| {
                    chunk.push_code(OpCode::DefineGlobal, line);
                    chunk.push(index as u8, line);
                });
            }
            None => {
                self.scope_mut().mark_local_initialized();
            }
        }
    }

    #[inline]
    fn function_declaration(&mut self) -> CompilationResult {
        let global_index = self.parse_variable("Expect function name.")?;
        let line = self.current_token().line;
        if global_index.is_none() {
            self.scope_mut().mark_local_initialized();
        }
        self.compile_function(FunctionType::Function)?;
        self.define_variable(global_index, line);
        Ok(())
    }

    fn compile_function(&mut self, function_type: FunctionType) -> CompilationResult {
        let function_name = self.intern_string();
        let function_name_line = self.previous_token().line;

        let compiler_context = CompilerContext {
            scanner: Rc::clone(&self.scanner),
            source: self.source,
            parse_rules: self.parse_rules,
            interned_strings: Rc::clone(&self.interned_strings),
            previous_token: self.previous_token.clone(),
            current_token: self.current_token.clone(),
            enclosing_scope: Rc::clone(&self.scope),
            is_inside_class: self.is_inside_class,
        };
        let mut compiler = Compiler::new(compiler_context);

        let token = Token {
            token_type: function_type.initial_local_variable_token_type(),
            lexeme: None,
            line: 0,
        };
        compiler.scope.as_ref().borrow_mut().add_local(token)?;

        let arity = compiler.parse_function()?;
        let line = self.previous_token().line;
        compiler.emit_return(line, &function_type);

        self.previous_token = compiler.previous_token.clone();
        self.current_token = compiler.current_token.clone();

        let upvalue_count = compiler.scope().upvalues_size();
        let function = ObjectFunction {
            name: function_name,
            arity,
            upvalue_count,
            chunk: mem::replace(&mut compiler.chunk, Chunk::new()),
        };
        let constant_index = self.chunk.push_constant_to_pool(Value::Function(Rc::new(function)));
        self.chunk.push_code(OpCode::Closure, function_name_line);
        self.chunk.push(constant_index as u8, function_name_line);
        for upvalue in compiler.scope().upvalues_iter() {
            self.chunk.push(if upvalue.is_local { 1 } else { 0 }, function_name_line);
            self.chunk.push(upvalue.index, function_name_line);
        }
        Ok(())
    }

    fn parse_function(&mut self) -> Result<u8, CompileError> {
        self.scope_mut().begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after function name.")?;
        let arity = match self.current_token().token_type {
            TokenType::RightParen => 0u8,
            _ => self.parse_function_parameters()?
        };
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;
        self.consume(TokenType::LeftBrace, "Expect '{' before function body.")?;
        self.block_statement()?;
        Ok(arity)
    }

    fn parse_function_parameters(&mut self) -> Result<u8, CompileError> {
        let mut arity: u8 = 0;
        loop {
            if arity == u8::MAX {
               return Err(CompileError::make_from_token(
                   self.current_token(),
                   "Can't have more than 255 parameters."
               ))
            }
            arity += 1;
            let index = self.parse_variable("Expect parameter name.")?;
            self.define_variable(index, self.previous_token().line);
            if self.current_token().token_type == TokenType::Comma {
                self.advance()?;
            } else {
                break;
            }
        }
        Ok(arity)
    }

    #[inline]
    fn class_declaration(&mut self) -> CompilationResult {
        self.consume(TokenType::Identifier, "Expect class name.")?;
        let name = self.intern_string();
        let constant_index = self.chunk.push_constant_to_pool(Value::String(name));
        let line = self.previous_token().line;
        if self.scope().is_global_scope() {
            self.push_code(OpCode::Class);
            self.chunk.push(constant_index as u8, line);
            self.define_variable(Some(constant_index), line);
        } else {
            self.declare_local_variable()?;
            self.push_code(OpCode::Class);
            self.chunk.push(constant_index as u8, line);
            self.define_variable(None, line);
        }
        let prev_is_inside_class = self.is_inside_class;
        self.is_inside_class = true;

        // TODO: Will break with inheritance
        self.variable(false)?;
        self.consume(TokenType::LeftBrace, "Expect '{' before class body.")?;
        loop {
            match self.current_token().token_type {
                TokenType::RightBrace | TokenType::Eof => {
                    break;
                }
                _ => {
                    self.method()?;
                }
            }
        }
        self.consume(TokenType::RightBrace, "Expect '}' after class body.")?;
        self.chunk.push_code(OpCode::Pop, line);
        self.is_inside_class = prev_is_inside_class;
        Ok(())
    }

    #[inline]
    fn method(&mut self) -> CompilationResult {
        self.consume(TokenType::Identifier, "Expect method name.")?;
        let name = self.intern_string();
        let is_initializer = &name.value == INIT_KEYWORD;
        let constant_index = self.chunk.push_constant_to_pool(Value::String(name));
        let line = self.previous_token().line;
        self.compile_function(FunctionType::Method(is_initializer))?;
        self.chunk.push_code(OpCode::Method, line);
        self.chunk.push(constant_index as u8, line);
        Ok(())
    }

    #[inline]
    fn print_statement(&mut self) -> CompilationResult {
        self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        self.push_code(OpCode::Print);
        Ok(())
    }

    fn parse_block(&mut self) -> CompilationResult {
        self.advance()?;
        self.scope_mut().begin_scope();
        let result = self.block_statement();
        self.end_scope();
        result
    }

    fn block_statement(&mut self) -> CompilationResult {
        loop {
            match self.current_token().token_type {
                TokenType::RightBrace => {
                    self.advance()?;
                    return Ok(())
                },
                TokenType::Eof => {
                    return Err(
                        CompileError::make_from_token(self.current_token(), "Expect '}' after block.")
                    )
                }
                _ => {
                    self.declaration()?;
                }
            }
        }
    }

    fn while_statement(&mut self) -> CompilationResult {
        let loop_start = self.current_chunk_size();
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;

        let condition_line = self.current_token().line;
        let then_jump = self.emit_jump(OpCode::JumpIfFalse, condition_line);
        self.modify_chunk(|chunk| chunk.push_code(OpCode::Pop, condition_line));

        let previous_loop_context = self.loop_context;
        let locals_depth = self.scope().current_scope_depth();
        self.loop_context = Some(LoopContext {
            start_index: loop_start,
            locals_depth,
        });
        self.statement()?;
        self.loop_context = previous_loop_context;

        self.emit_loop(loop_start, condition_line)?;
        self.patch_jump(then_jump)?;
        self.modify_chunk(|chunk| chunk.push_code(OpCode::Pop, condition_line));
        Ok(())
    }

    fn for_statement(&mut self) -> CompilationResult {
        self.scope_mut().begin_scope();
        let statement_line = self.current_token().line;
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;
        self.initializer_clause()?;

        let previous_loop_context = self.loop_context;
        let mut loop_start = self.current_chunk_size();
        let locals_depth = self.scope().current_scope_depth();
        self.loop_context = Some(LoopContext {
            start_index: loop_start,
            locals_depth,
        });
        let exit_jump = self.condition_clause()?;
        self.increment_clause(&mut loop_start)?;

        let locals_depth = self.scope().current_scope_depth();
        self.loop_context = Some(LoopContext {
            start_index: loop_start,
            locals_depth,
        });
        self.statement()?;
        self.loop_context = previous_loop_context;
        self.emit_loop(loop_start, statement_line)?;

        if let Some(exit_jump) = exit_jump {
            self.patch_jump(exit_jump)?;
            self.modify_chunk(|chunk| chunk.push_code(OpCode::Pop, statement_line));
        }
        self.end_scope();
        Ok(())
    }

    #[inline]
    fn initializer_clause(&mut self) -> CompilationResult {
        match self.current_token().token_type {
            TokenType::Semicolon => self.advance(),
            TokenType::Var => {
                self.advance()?;
                self.variable_declaration()
            }
            _ => self.expression_statement()
        }
    }

    #[inline]
    fn condition_clause(&mut self) -> Result<Option<usize>, CompileError> {
        if self.current_token().token_type == TokenType::Semicolon {
            return self.advance().map(|_| None);
        }
        self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;
        let line = self.current_token().line;
        let jump = self.emit_jump(OpCode::JumpIfFalse, line);
        self.modify_chunk(|chunk| chunk.push_code(OpCode::Pop, line));
        Ok(Some(jump))
    }

    #[inline]
    fn increment_clause(&mut self, loop_start: &mut usize) -> CompilationResult {
        if self.current_token().token_type == TokenType::RightParen {
            return self.advance();
        }
        let line = self.current_token().line;
        let body_jump = self.emit_jump(OpCode::Jump, line);
        let increment_start = self.current_chunk_size();
        self.expression()?;
        self.modify_chunk(|chunk| chunk.push_code(OpCode::Pop, line));
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        self.emit_loop(*loop_start, line)?;
        *loop_start = increment_start;
        self.patch_jump(body_jump)?;
        Ok(())
    }

    fn if_statement(&mut self) -> CompilationResult {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;

        let if_condition_line = self.current_token().line;
        let then_jump = self.emit_jump(OpCode::JumpIfFalse, if_condition_line);
        self.modify_chunk(|chunk| chunk.push_code(OpCode::Pop, if_condition_line));
        self.statement()?;

        let else_jump = self.emit_jump(OpCode::Jump, self.current_token().line);
        self.patch_jump(then_jump)?;
        self.modify_chunk(|chunk| chunk.push_code(OpCode::Pop, if_condition_line));

        if self.current_token().token_type == TokenType::Else {
            self.advance()?;
            self.statement()?;
        }
        self.patch_jump(else_jump)
    }

    #[inline]
    fn emit_loop(&mut self, loop_start: usize, line: usize) -> CompilationResult {
        self
            .modify_chunk(|chunk| {
                chunk.push_code(OpCode::Loop, line);
                let jump = chunk.codes.length - loop_start + 2;
                if jump > u16::MAX as usize {
                    Err("Too much code to jump over.")
                } else {
                    let jump = jump as u16;
                    chunk.push(((jump >> 8u8) & 0xff) as u8, line);
                    chunk.push((jump & 0xff) as u8, line);
                    Ok(())
                }
            })
            .map_err(|message| {
                CompileError::make_from_token(self.current_token(), message)
            })
    }

    #[inline]
    fn emit_jump(&mut self, op_code: OpCode, line: usize) -> usize {
        self.modify_chunk(|chunk| {
            chunk.push_code(op_code, line);
            chunk.push(0, line);
            chunk.push(0, line);
            chunk.codes.length - 2
        })
    }

    #[inline]
    fn patch_jump(&mut self, offset: usize) -> CompilationResult {
        self
            .modify_chunk(|chunk| {
                let jump = chunk.codes.length - offset - 2;
                if jump > u16::MAX as usize {
                    Err("Too much code to jump over.")
                } else {
                    let jump = jump as u16;
                    chunk.codes[offset] = ((jump >> 8u8) & 0xff) as u8;
                    chunk.codes[offset + 1] = (jump & 0xff) as u8;
                    Ok(())
                }
            })
            .map_err(|message| {
                CompileError::make_from_token(self.current_token(), message)
            })
    }

    #[inline]
    fn continue_statement(&mut self) -> CompilationResult {
        self.consume(TokenType::Semicolon, "Expect ';' after continue statement.")?;
        let token = self.previous_token();
        match self.loop_context {
            Some(context) => {
                let line = token.line;
                let locals_count = self.scope_mut().remove_to_scope(context.locals_depth + 1);
                for _ in 0..locals_count {
                    self.modify_chunk(|chunk| chunk.push_code(OpCode::Pop, line));
                }
                self.emit_loop(context.start_index, line)
            }
            None => {
                let token = self.previous_token();
                Err(CompileError::make_from_token(token, "Can't use 'continue' outside of a loop."))
            }
        }
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

        let can_assign = precedence <= Precedence::Assignment;
        match rule.parse_type.prefix() {
            Some(func) => func(self, can_assign),
            None => Err(CompileError::make_from_token(token, "Expect expression."))
        }?;
        let precedence_int: u8 = precedence as u8;
        loop {
            let current_rule = self.parse_rule(&self.current_token().token_type);
            if (current_rule.precedence as u8) < precedence_int {
                break;
            }
            self.advance()?;
            let token = self.previous_token();
            let rule = self.parse_rule(&token.token_type);
            match rule.parse_type.infix() {
                Some(func) => func(self, can_assign),
                None => Err(CompileError::make_from_token(token, "Expect expression."))
            }?;
        };
        if can_assign && self.current_token().token_type == TokenType::Equal {
            self.advance()?;
            Err(CompileError::make_from_token(self.previous_token(), "Invalid assignment target."))
        } else {
            Ok(())
        }
    }

    #[inline]
    fn grouping(&mut self, _can_assign: bool) -> CompilationResult {
        self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after expression.")
    }

    fn unary(&mut self, _can_assign: bool) -> CompilationResult {
        let previous_token = self.previous_token();
        let line = previous_token.line;
        let token_type = previous_token.token_type;
        self.parse_precedence(Precedence::Unary)?;
        self.modify_chunk(|chunk| {
            match token_type {
                TokenType::Minus => chunk.push_code(OpCode::Negate, line),
                TokenType::Bang => chunk.push_code(OpCode::Not, line),
                _ => {}
            }
        });
        Ok(())
    }

    fn binary(&mut self, _can_assign: bool) -> CompilationResult {
        let previous_token = self.previous_token();
        let token_type = previous_token.token_type;
        let token_line = previous_token.line;
        let rule = self.parse_rule(&token_type);
        let precedence = Precedence::try_from((rule.precedence as u8) + 1).unwrap();
        self.parse_precedence(precedence)?;
        self.modify_chunk(|chunk| {
            match token_type {
                TokenType::Plus => chunk.push_code(OpCode::Add, token_line),
                TokenType::Minus => chunk.push_code(OpCode::Subtract, token_line),
                TokenType::Star => chunk.push_code(OpCode::Multiply, token_line),
                TokenType::Slash => chunk.push_code(OpCode::Divide, token_line),
                TokenType::BangEqual => {
                    chunk.push_code(OpCode::Equal, token_line);
                    chunk.push_code(OpCode::Not, token_line);
                }
                TokenType::EqualEqual => chunk.push_code(OpCode::Equal, token_line),
                TokenType::Greater => chunk.push_code(OpCode::Greater, token_line),
                TokenType::GreaterEqual => {
                    chunk.push_code(OpCode::Less, token_line);
                    chunk.push_code(OpCode::Not, token_line);
                }
                TokenType::Less => chunk.push_code(OpCode::Less, token_line),
                TokenType::LessEqual => {
                    chunk.push_code(OpCode::Greater, token_line);
                    chunk.push_code(OpCode::Not, token_line);
                }
                _ => {}
            };
        });
        Ok(())
    }

    fn dot(&mut self, can_assign: bool) -> CompilationResult {
        self.consume(TokenType::Identifier, "Expect property name after '.'.")?;
        let name = self.intern_string();
        let constant_index = self.chunk.push_constant_to_pool(Value::String(name)) as u8;
        let line = self.current_token().line;
        if can_assign && self.current_token().token_type == TokenType::Equal {
            self.advance()?;
            self.expression()?;
            self.chunk.push_code(OpCode::SetProperty, line);
            self.chunk.push(constant_index, line);
            Ok(())
        } else {
            self.chunk.push_code(OpCode::GetProperty, line);
            self.chunk.push(constant_index, line);
            Ok(())
        }
    }

    fn and_operator(&mut self, _can_assign: bool) -> CompilationResult {
        let line = self.current_token().line;
        let jump = self.emit_jump(OpCode::JumpIfFalse, line);
        self.modify_chunk(|chunk| chunk.push_code(OpCode::Pop, line));

        self.parse_precedence(Precedence::And)?;
        self.patch_jump(jump)
    }

    fn or_operator(&mut self, _can_assign: bool) -> CompilationResult {
        let line = self.current_token().line;
        let else_jump = self.emit_jump(OpCode::JumpIfFalse, line);
        let end_jump = self.emit_jump(OpCode::Jump, line);

        self.patch_jump(else_jump)?;
        self.modify_chunk(|chunk| chunk.push_code(OpCode::Pop, line));

        self.parse_precedence(Precedence::Or)?;
        self.patch_jump(end_jump)
    }

    fn emit_number(&mut self, _can_assign: bool) -> CompilationResult {
        let number: f32 = self.previous_token().lexeme
            .as_ref()
            .expect("Only EOF token can not have a lexeme")
            .make_slice(self.source)
            .parse()
            .expect("Invalid number parsed");
        let line = self.previous_token().line;
        self.modify_chunk(|chunk| chunk.add_constant(Value::Number(number), line));
        Ok(())
    }

    fn literal(&mut self, _can_assign: bool) -> CompilationResult {
        let previous_token = self.previous_token();
        let line = previous_token.line;
        let token_type = previous_token.token_type;
        self.modify_chunk(|chunk| {
            match token_type {
                TokenType::True => chunk.push_code(OpCode::True, line),
                TokenType::False => chunk.push_code(OpCode::False, line),
                TokenType::Nil => chunk.push_code(OpCode::Nil, line),
                _ => {}
            }
        });
        Ok(())
    }

    fn string(&mut self, _can_assign: bool) -> CompilationResult {
        let object = self.intern_string();
        let line = self.previous_token().line;
        self.modify_chunk(|chunk| {
            chunk.add_constant(Value::String(object), line);
        });
        Ok(())
    }

    fn variable(&mut self, can_assign: bool) -> CompilationResult {
        let (set_code, get_code, index) = self.variable_operations()?;
        if can_assign && self.current_token().token_type == TokenType::Equal {
            self.advance()?;
            self.expression()?;
            let line = self.previous_token().line;
            self.modify_chunk(|chunk| {
                chunk.push_code(set_code, line);
                chunk.push(index, line);
            });
        } else {
            let line = self.previous_token().line;
            self.modify_chunk(|chunk| {
                chunk.push_code(get_code, line);
                chunk.push(index, line);
            });
        }
        Ok(())
    }

    fn function_call(&mut self, _can_assign: bool) -> CompilationResult {
        let line = self.current_token().line;
        let arguments_count = self.parse_arguments()?;
        self.chunk.push_code(OpCode::Call, line);
        self.chunk.push(arguments_count, line);
        Ok(())
    }

    #[inline]
    fn parse_arguments(&mut self) -> Result<u8, CompileError> {
        let mut arguments_count = 0u8;
        loop {
            match self.current_token().token_type {
                TokenType::RightParen => {
                    break;
                }
                TokenType::Comma => {
                    self.advance()?;
                }
                _ => {}
            }
            if arguments_count == u8::MAX {
                let token = self.current_token();
                return Err(CompileError::make_from_token(token, "Can't have more than 255 arguments."));
            }
            arguments_count += 1;
            self.expression()?;
        }
        self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
        Ok(arguments_count)
    }

    #[inline]
    fn variable_operations(&mut self) -> Result<(OpCode, OpCode, u8), CompileError> {
        let local_index = self.scope().find_local(self.previous_token(), self.source)?;
        match local_index {
            Some(index) => Ok((OpCode::SetLocal, OpCode::GetLocal, index)),
            None => {
                if let Some(upvalue_index) = self.scope_mut().resolve_upvalue(self.previous_token(), self.source)? {
                    return Ok((OpCode::SetUpvalue, OpCode::GetUpvalue, upvalue_index));
                }
                let object = self.intern_string();
                let index = self.string_constants.find(&object).copied();
                let index = match index {
                    Some(index) => index,
                    None => {
                        let string = Rc::clone(&object);
                        let index = self.modify_chunk(|chunk| {
                            chunk.push_constant_to_pool(Value::String(object)) as u8
                        });
                        self.string_constants.insert(string, index);
                        index
                    }
                };
                Ok((OpCode::SetGlobal, OpCode::GetGlobal, index))
            }
        }
    }

    #[inline]
    fn this(&mut self, _can_assign: bool) -> CompilationResult {
        if self.is_inside_class == false {
            Err(CompileError::make_from_token(self.previous_token(), "Can't use 'this' outside of a class."))
        } else {
            self.variable(false)
        }
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

    fn emit_return(&mut self, line: usize, function_type: &FunctionType) {
        if function_type.is_initializer() {
            self.chunk.push_code(OpCode::GetLocal, line);
            self.chunk.push(0u8, line);
        } else {
            self.chunk.push_code(OpCode::Nil, line);
        }
        self.modify_chunk(|chunk| chunk.push_code(OpCode::Return, line));
    }

    fn end_scope(&mut self) {
        let line = self.current_token().line;
        let op_codes = self.scope_mut().end_scope();
        for op_code in op_codes {
            self.chunk.push_code(op_code, line);
        }
    }

    #[inline]
    fn parse_rule(&self, token_type: &TokenType) -> &ParseRule<'a> {
        &self.parse_rules[*token_type as usize]
    }

    #[inline]
    fn push_code(&mut self, code: OpCode) {
        let line = self.current_token().line;
        self.modify_chunk(|chunk| chunk.push_code(code, line));
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

    pub fn make_parse_rules<'c>() -> [ParseRule<'c>; 39] {
        return [
            ParseRule {
                parse_type: ParseType::Both {
                    prefix: Compiler::grouping,
                    infix: Compiler::function_call
                },
                precedence: Precedence::Call
            },                                                                       // TokenType::LeftParen
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::RightParen
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::LeftBrace
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::RightBrace
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Comma
            ParseRule {
                parse_type: ParseType::Infix(Compiler::dot),
                precedence: Precedence::Call
            }, // TokenType::Dot
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
            ParseRule {
                parse_type: ParseType::Infix(Compiler::and_operator),
                precedence: Precedence::And
            }, // TokenType::And
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
            ParseRule {
                parse_type: ParseType::Infix(Compiler::or_operator),
                precedence: Precedence::Or
            }, // TokenType::Or
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Print
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Return
            ParseRule { parse_type: ParseType::None, precedence: Precedence::None }, // TokenType::Super
            ParseRule {
                parse_type: ParseType::Prefix(Compiler::this),
                precedence: Precedence::None
            }, // TokenType::This
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
    pub fn make_from_token(token: &Token, message: &'static str) -> Self {
        CompileError::TokenError {
            line: token.line,
            lexeme: token.lexeme,
            message
        }
    }
}

#[derive(Clone, Copy)]
struct LoopContext {
    start_index: usize,
    locals_depth: u8,
}

pub type CompilationResult = Result<(), CompileError>;

enum FunctionType {
    Function,
    Method(bool)
}

impl FunctionType {
    fn initial_local_variable_token_type(&self) -> TokenType {
        match self {
            FunctionType::Function => TokenType::Nil,
            FunctionType::Method(_) => TokenType::This,
        }
    }
    fn is_initializer(&self) -> bool {
        match self {
            FunctionType::Function => false,
            FunctionType::Method(is_initializer) => *is_initializer,
        }
    }

}
