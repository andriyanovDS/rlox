use std::iter::Rev;
use std::slice::Iter;
use super::compiler::{CompilationResult, CompileError};
use super::token::{Token, TokenType};

const STACK_SIZE: usize = u8::MAX as usize + 1;
const NOT_INITIALIZED: Local = Local {
    token: Token {
        token_type: TokenType::Nil,
        lexeme: None,
        line: 0
    },
    depth: 0
};

struct Local {
    token: Token,
    depth: u8,
}

pub struct Scope {
    locals: [Local; STACK_SIZE],
    locals_count: u8,
    scope_depth: u8,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            locals: [NOT_INITIALIZED; STACK_SIZE],
            locals_count: 0,
            scope_depth: 0,
        }
    }

    #[inline]
    pub fn is_global_scope(&self) -> bool {
        self.scope_depth == 0
    }

    #[inline]
    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    #[inline]
    pub fn end_scope(&mut self) -> u8 {
        let local_count = self.locals_iter()
            .take_while(|v| v.depth == self.scope_depth)
            .fold(0u8, |acc, _| acc + 1);
        self.scope_depth -= 1;
        self.locals_count -= local_count;
        local_count
    }

    #[inline]
    pub fn find_local(&self, token: &Token, source: &str) -> Result<Option<u8>, CompileError> {
        if self.locals_count == 0 {
            return Ok(None);
        }
        let lexeme = token.lexeme.as_ref().unwrap().make_slice(source);
        let end_index = self.locals_count - 1;
        for (index, local) in self.locals_iter().enumerate() {
            let stored_lexeme = local.token.lexeme.as_ref().unwrap().make_slice(source);
            if stored_lexeme != lexeme {
                continue;
            }
            if local.depth == 0 {
                let message = "Can't read local variable in its own initializer.";
                return Err(CompileError::make_from_token(token, message));
            }
            return Ok(Some(end_index - index as u8));
        }
        Ok(None)
    }

    #[inline]
    pub fn is_redeclaration(&self, token: &Token, source: &str) -> bool {
        let lexeme = token.lexeme.as_ref().unwrap().make_slice(source);
        for local in self.locals_iter() {
            if local.depth != 0 && local.depth < self.scope_depth {
                return false;
            }
            let stored_lexeme = local.token.lexeme.as_ref().unwrap().make_slice(source);
            if stored_lexeme == lexeme {
                return true;
            }
        }
        false
    }

    #[inline]
    pub fn add_local(&mut self, token: Token) -> CompilationResult {
        if self.locals_count == u8::MAX {
            return Err(CompileError::make_from_token(&token, "Too many local variables in function."));
        }
        let index = self.locals_count;
        self.locals[index as usize] = Local {
            token,
            depth: 0,
        };
        self.locals_count += 1;
        Ok(())
    }

    #[inline]
    pub fn mark_local_initialized(&mut self) {
        let local = &mut self.locals[(self.locals_count - 1) as usize];
        local.depth = self.scope_depth;
    }

    #[inline]
    fn locals_iter(&self) -> Rev<Iter<Local>> {
        self.locals[0..self.locals_count as usize].iter().rev()
    }
}