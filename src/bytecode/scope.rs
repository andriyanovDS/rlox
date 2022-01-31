use crate::bytecode::compiler::{CompilationResult, CompileError};
use crate::bytecode::token::{Token, TokenType};

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
        let local_count = self.locals[0..self.locals_count as usize]
            .iter()
            .rev()
            .take_while(|v| v.depth == self.scope_depth)
            .fold(0u8, |acc, _| acc + 1);
        self.scope_depth -= 1;
        self.locals_count -= local_count;
        local_count
    }

    pub fn is_redeclaration(&self, token: &Token, source: &str) -> bool {
        let lexeme = token.lexeme.as_ref().unwrap().make_slice(source);
        for local in self.locals[0..self.locals_count as usize].iter().rev() {
            if local.depth < self.scope_depth {
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
            depth: self.scope_depth,
        };
        self.locals_count += 1;
        Ok(())
    }
}