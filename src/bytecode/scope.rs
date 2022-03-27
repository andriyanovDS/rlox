use std::cell::RefCell;
use std::iter::Rev;
use std::rc::Rc;
use std::slice::Iter;
use super::upvalue::{Upvalues, UpvaluesRefIterator};
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
    enclosing_scope: Option<Rc<RefCell<Scope>>>,
    upvalues: Upvalues,
    locals: [Local; STACK_SIZE],
    locals_count: u8,
    scope_depth: u8,
}

impl Scope {
    pub const fn new(enclosing_scope: Option<Rc<RefCell<Scope>>>) -> Self {
        Self {
            enclosing_scope,
            upvalues: Upvalues::new(),
            locals: [NOT_INITIALIZED; STACK_SIZE],
            locals_count: 0,
            scope_depth: 0,
        }
    }

    pub fn take_enclosing_scope(&mut self) -> Option<Rc<RefCell<Scope>>> {
        self.enclosing_scope.take()
    }

    pub fn upvalues_size(&self) -> u8 {
        self.upvalues.size()
    }

    pub fn upvalues_iter(&self) -> UpvaluesRefIterator {
        (&self.upvalues).into_iter()
    }

    pub fn current_scope_depth(&self) -> u8 { self.scope_depth }

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
    pub fn remove_to_scope(&mut self, scope_depth: u8) -> u8 {
        let local_count = self.locals_iter()
            .take_while(|v| v.depth >= scope_depth)
            .fold(0u8, |acc, _| acc + 1);
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
    pub fn resolve_upvalue(
        &mut self,
        token: &Token,
        source: &str
    ) -> Result<Option<u8>, CompileError> {
        match &self.enclosing_scope {
            None => Ok(None),
            Some(scope) => {
                let mut scope = scope.as_ref().borrow_mut();
                match scope.find_local(token, source)? {
                    Some(index) => {
                        drop(scope);
                        self.add_upvalue(token, index, true)
                    },
                    None => {
                        match scope.resolve_upvalue(token, source)? {
                            Some(index) => {
                                drop(scope);
                                self.add_upvalue(token, index, false)
                            }
                            None => Ok(None)
                        }
                    }
                }
            }
        }
    }

    #[inline]
    fn add_upvalue(&mut self, token: &Token, index: u8, is_local: bool) -> Result<Option<u8>, CompileError> {
        match self.upvalues.push(index, is_local) {
            None => Err(CompileError::make_from_token(token, "Too many closure variables in function.")),
            Some(index) => Ok(Some(index))
        }
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
        assert!(self.scope_depth > 0);
        let local = &mut self.locals[(self.locals_count - 1) as usize];
        local.depth = self.scope_depth;
    }

    #[inline]
    fn locals_iter(&self) -> Rev<Iter<Local>> {
        self.locals[0..self.locals_count as usize].iter().rev()
    }
}
