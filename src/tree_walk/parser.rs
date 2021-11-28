use super::error::Error;
use super::expression::{Expression, LiteralExpression, VariableExpression};
use super::lox_function::LoxFunction;
use super::statement::Statement;
use super::token::Token;
use super::token_type::{
    Delimiter, ExpressionOperatorTokenType, KeywordTokenType, LiteralTokenType,
    SingleCharTokenType, TokenType,
};
use std::iter::Peekable;
use std::rc::Rc;
use std::slice::Iter;

pub struct Parser<'a> {
    tokens_iter: Peekable<Iter<'a, Token>>,
    current: Option<&'a Token>,
}

type ParseStmtResult = Result<Statement, ParseError>;
type ParseExprResult = Result<Expression, ParseError>;

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens_iter: tokens.iter().peekable(),
            current: None,
        }
    }

    pub fn parse(&mut self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = Vec::new();
        let mut had_error = false;
        loop {
            match self.declaration() {
                Ok(expression) => {
                    statements.push(expression);
                }
                Err(error) if error.token.token_type == TokenType::Eof => break,
                Err(error) => {
                    self.synchronize();
                    had_error = true;
                    eprintln!("{}", error.description());
                }
            }
        }
        if had_error {
            Vec::new()
        } else {
            statements
        }
    }

    fn declaration(&mut self) -> ParseStmtResult {
        match self.tokens_iter.peek().unwrap().token_type {
            TokenType::Keyword(KeywordTokenType::Class) => {
                self.advance();
                self.class_statement()
            }
            TokenType::Keyword(KeywordTokenType::Fun) => {
                self.advance();
                self.function_statement()
            }
            TokenType::Keyword(KeywordTokenType::Var) => {
                self.advance();
                self.variable_statement()
            }
            _ => self.statement(),
        }
    }

    fn statement(&mut self) -> ParseStmtResult {
        match self.tokens_iter.peek().unwrap().token_type {
            TokenType::Keyword(KeywordTokenType::For) => {
                self.advance();
                self.for_statement()
            }
            TokenType::Keyword(KeywordTokenType::If) => {
                self.advance();
                self.if_statement()
            }
            TokenType::Keyword(KeywordTokenType::Print) => {
                self.advance();
                self.print_statement()
            }
            TokenType::Keyword(KeywordTokenType::Return) => {
                self.advance();
                self.return_statement()
            }
            TokenType::Keyword(KeywordTokenType::While) => {
                self.advance();
                self.while_statement()
            }
            TokenType::OpenDelimiter(Delimiter::Brace) => {
                self.advance();
                self.block().map(Statement::Block)
            }
            _ => self.expression_statement(),
        }
    }

    fn block(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements: Vec<Statement> = Vec::new();
        loop {
            match self.tokens_iter.peek().map(|token| &token.token_type) {
                Some(TokenType::CloseDelimiter(Delimiter::Brace)) => {
                    self.advance();
                    return Ok(statements);
                }
                Some(TokenType::Eof) => {
                    self.advance();
                    return Err(self.make_error("Expect '}' after block."));
                }
                _ => {
                    let statement = self.declaration()?;
                    statements.push(statement);
                }
            }
        }
    }

    fn class_statement(&mut self) -> ParseStmtResult {
        let name = self.consume_identifier(|| "Expect class name.")?;
        let superclass = self
            .parse_superclass()?
            .map(|name| VariableExpression {
                name,
                token: self.current.unwrap().clone()
            });

        if !self.next_matches_one(TokenType::OpenDelimiter(Delimiter::Brace)) {
            return Err(self.make_error("Expect '{' before class methods"));
        }
        self.advance();
        let mut methods = Vec::new();
        let mut static_methods = Vec::new();
        loop {
            if self.next_matches_one(TokenType::CloseDelimiter(Delimiter::Brace)) {
                self.advance();
                return Ok(Statement::Class { name, methods, static_methods, superclass });
            }
            if self.next_matches_one(TokenType::Keyword(KeywordTokenType::Class)) {
                self.advance();
                let method = self.parse_function()?;
                static_methods.push(method);
            } else {
                let method = self.parse_function()?;
                methods.push(method);
            }
        }
    }

    fn parse_superclass(&mut self) -> Result<Option<String>, ParseError> {
        let less_token_type = TokenType::ExpressionOperator(ExpressionOperatorTokenType::Less);
        if !self.next_matches_one(less_token_type) {
            return Ok(None)
        }
        self.advance();
        self.parse_variable_name()
            .ok_or_else(|| {
                self.make_error("Expect superclass name.")
            })
            .map(Some)
    }

    fn function_statement(&mut self) -> ParseStmtResult {
        self.parse_function().map(Statement::Function)
    }

    fn parse_function(&mut self) -> Result<Rc<LoxFunction>, ParseError> {
        let name = self.consume_identifier(|| "Expect function name.")?;
        self.advance();
        let parameters = self.parse_function_parameters()?;
        self.advance_when_match(
            TokenType::OpenDelimiter(Delimiter::Brace),
            |parser| {
                let body = parser.block()?;
                let lox_func = LoxFunction::new(name, parameters, body);
                Ok(Rc::new(lox_func))
            },
            |parser| Err(parser.make_error("Expect '{' before function body.")),
        )
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<String>, ParseError> {
        let mut parameters = Vec::new();
        if self.next_matches_one(TokenType::CloseDelimiter(Delimiter::Paren)) {
            self.advance();
            return Ok(parameters);
        }
        loop {
            let parameter = self.consume_identifier(|| "Expect parameter name.")?;
            parameters.push(parameter);
            match &self.tokens_iter.peek().unwrap().token_type {
                TokenType::SingleChar(SingleCharTokenType::Comma) => {
                    self.advance();
                }
                TokenType::CloseDelimiter(Delimiter::Paren) => {
                    self.advance();
                    return Ok(parameters);
                }
                _ => return Err(self.make_error("Expect ')' after parameters.")),
            }
        }
    }

    fn variable_statement(&mut self) -> ParseStmtResult {
        self.parse_variable_name()
            .map(|name| {
                self.make_variable_stmt(name.to_string())
                    .and_then(|stmt| self.check_semicolon_after_stmt(stmt))
            })
            .unwrap_or_else(|| {
                let token = self.tokens_iter.peek().unwrap();
                Err(ParseError {
                    token: (*token).clone(),
                    message: "Expect variable name.",
                })
            })
    }

    fn parse_variable_name(&mut self) -> Option<String> {
        let token = self.tokens_iter.peek().unwrap();
        if let TokenType::Literal(LiteralTokenType::Identifier(ref name)) = token.token_type {
            self.advance();
            Some(name.to_string())
        } else {
            None
        }
    }

    fn make_variable_stmt(&mut self, name: String) -> ParseStmtResult {
        let equal_type = TokenType::ExpressionOperator(ExpressionOperatorTokenType::Equal);
        if self.next_matches_one(equal_type) {
            self.advance();
            let right = self.expression()?;
            Ok(Statement::Variable {
                name,
                value: Some(right),
            })
        } else {
            Ok(Statement::Variable { name, value: None })
        }
    }

    fn for_statement(&mut self) -> ParseStmtResult {
        self.advance_when_match(
            TokenType::OpenDelimiter(Delimiter::Paren),
            |_| Ok(()),
            |parser| Err(parser.make_error("Expect '(' after 'for'.")),
        )?;
        let initializer = self.parse_for_initializer()?;
        let condition = self.parse_for_expression(
            TokenType::SingleChar(SingleCharTokenType::Semicolon),
            || "Expect ';' after loop condition.",
        )?;
        let increment = self
            .parse_for_expression(TokenType::CloseDelimiter(Delimiter::Paren), || {
                "Expect ')' after for clauses."
            })?;
        let body = self.statement()?;
        let loop_body: Statement = match increment {
            Some(expr) => Statement::Block(vec![body, Statement::Expression(expr)]),
            None => Statement::Block(vec![body]),
        };
        let loop_condition = condition.unwrap_or(Expression::Literal(LiteralExpression::True));
        let while_loop = Statement::While {
            condition: loop_condition,
            body: Box::new(loop_body),
        };
        match initializer {
            Some(stmt) => Ok(Statement::Block(vec![stmt, while_loop])),
            None => Ok(while_loop),
        }
    }

    fn parse_for_initializer(&mut self) -> Result<Option<Statement>, ParseError> {
        let next_token = self.tokens_iter.peek().unwrap();
        match next_token.token_type {
            TokenType::SingleChar(SingleCharTokenType::Semicolon) => {
                self.advance();
                Ok(None)
            }
            TokenType::Keyword(KeywordTokenType::Var) => {
                self.advance();
                self.variable_statement().map(Some)
            }
            _ => self.expression_statement().map(Some),
        }
    }

    fn parse_for_expression<EF>(
        &mut self,
        token_type: TokenType,
        err_message_provider: EF,
    ) -> Result<Option<Expression>, ParseError>
    where
        EF: Fn() -> &'static str,
    {
        if self.tokens_iter.peek().unwrap().token_type == token_type {
            self.advance();
            Ok(None)
        } else {
            let expression = self.expression()?;
            self.advance_when_match(
                token_type,
                move |_| Ok(Some(expression)),
                |parser| Err(parser.make_error(err_message_provider())),
            )
        }
    }

    fn if_statement(&mut self) -> ParseStmtResult {
        let condition: Expression = self.advance_when_match(
            TokenType::OpenDelimiter(Delimiter::Paren),
            Parser::expression,
            |parser| Err(parser.make_error("Expect '(' after 'if'.")),
        )?;
        let then_branch: Statement = self.advance_when_match(
            TokenType::CloseDelimiter(Delimiter::Paren),
            Parser::statement,
            |parser| Err(parser.make_error("Expect ')' after if condition.")),
        )?;
        let else_branch: Option<Statement> = self.advance_when_match(
            TokenType::Keyword(KeywordTokenType::Else),
            |parser| parser.statement().map(Some),
            |_| Ok(None),
        )?;
        Ok(Statement::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        })
    }

    fn print_statement(&mut self) -> ParseStmtResult {
        self.expression()
            .map(Statement::Print)
            .and_then(|stmt| self.check_semicolon_after_stmt(stmt))
    }

    fn return_statement(&mut self) -> ParseStmtResult {
        if self.next_matches_one(TokenType::SingleChar(SingleCharTokenType::Semicolon)) {
            self.advance();
            return Ok(Statement::Return(Expression::Literal(
                LiteralExpression::Nil,
            )));
        }
        let expression = self.expression()?;
        self.advance_when_match(
            TokenType::SingleChar(SingleCharTokenType::Semicolon),
            |_| Ok(Statement::Return(expression)),
            |parser| Err(parser.make_error("Expect ';' after return value.")),
        )
    }

    fn while_statement(&mut self) -> ParseStmtResult {
        let condition: Expression = self.advance_when_match(
            TokenType::OpenDelimiter(Delimiter::Paren),
            Parser::expression,
            |parser| Err(parser.make_error("Expect '(' after 'while'.")),
        )?;
        let body: Statement = self.advance_when_match(
            TokenType::CloseDelimiter(Delimiter::Paren),
            Parser::statement,
            |parser| Err(parser.make_error("Expect ')' after condition.")),
        )?;
        Ok(Statement::While {
            condition,
            body: Box::new(body),
        })
    }

    fn expression_statement(&mut self) -> ParseStmtResult {
        self.expression()
            .map(Statement::Expression)
            .and_then(|stmt| self.check_semicolon_after_stmt(stmt))
    }

    fn expression(&mut self) -> ParseExprResult {
        self.assignment()
    }

    fn assignment(&mut self) -> ParseExprResult {
        let left = self.logical_or()?;
        let equal_token_type = TokenType::ExpressionOperator(ExpressionOperatorTokenType::Equal);
        if self.next_matches_one(equal_token_type) {
            self.advance();
            let right = self.assignment()?;
            match left {
                Expression::Variable(expr) => {
                    Ok(Expression::Assignment(expr.token, Box::new(right)))
                },
                Expression::Get { name, expression } => {
                    Ok(Expression::Set { name, object: expression, value: Box::new(right) })
                },
                _ => Err(self.make_error("Invalid assignment target."))
            }
        } else {
            Ok(left)
        }
    }

    fn logical_or(&mut self) -> ParseExprResult {
        let token_type = TokenType::Keyword(KeywordTokenType::Or);
        self.find_binary_expression(Parser::logical_and, Expression::Logical, &[token_type])
    }

    fn logical_and(&mut self) -> ParseExprResult {
        let token_type = TokenType::Keyword(KeywordTokenType::And);
        self.find_binary_expression(Parser::equality, Expression::Logical, &[token_type])
    }

    fn equality(&mut self) -> ParseExprResult {
        let token_types = vec![
            TokenType::ExpressionOperator(ExpressionOperatorTokenType::NotEqual),
            TokenType::ExpressionOperator(ExpressionOperatorTokenType::EqualEqual),
        ];
        self.find_binary_expression(Parser::comparison, Expression::Binary, &token_types)
    }

    fn comparison(&mut self) -> ParseExprResult {
        let token_types = vec![
            TokenType::ExpressionOperator(ExpressionOperatorTokenType::Greater),
            TokenType::ExpressionOperator(ExpressionOperatorTokenType::GreaterEqual),
            TokenType::ExpressionOperator(ExpressionOperatorTokenType::Less),
            TokenType::ExpressionOperator(ExpressionOperatorTokenType::LessEqual),
        ];
        self.find_binary_expression(Parser::term, Expression::Binary, &token_types)
    }

    fn term(&mut self) -> ParseExprResult {
        let token_types = vec![
            TokenType::SingleChar(SingleCharTokenType::Minus),
            TokenType::SingleChar(SingleCharTokenType::Plus),
        ];
        self.find_binary_expression(Parser::factor, Expression::Binary, &token_types)
    }

    fn factor(&mut self) -> ParseExprResult {
        let token_types = vec![
            TokenType::SingleChar(SingleCharTokenType::Slash),
            TokenType::SingleChar(SingleCharTokenType::Star),
        ];
        self.find_binary_expression(Parser::unary, Expression::Binary, &token_types)
    }

    fn unary(&mut self) -> ParseExprResult {
        let token_types = vec![
            TokenType::SingleChar(SingleCharTokenType::Minus),
            TokenType::ExpressionOperator(ExpressionOperatorTokenType::Not),
        ];
        if self.next_matches_any(&token_types) {
            let operator = self.current.unwrap();
            self.advance();
            let right_expression = self.unary()?;
            Ok(Expression::Unary(
                operator.clone(),
                Box::new(right_expression),
            ))
        } else {
            self.find_call()
        }
    }

    fn find_call(&mut self) -> ParseExprResult {
        let mut expression = self.primary()?;
        loop {
            let next_token = self.tokens_iter.peek().unwrap();
            match next_token.token_type {
                TokenType::OpenDelimiter(Delimiter::Paren) => {
                    self.advance();
                    expression = self.finish_call(expression)?;
                },
                TokenType::SingleChar(SingleCharTokenType::Dot) => {
                    self.advance();
                    let name = self.consume_identifier(|| "Expect property name after '.'.")?;
                    expression = Expression::Get { name, expression: Box::new(expression) };
                }
                _ => {
                    return Ok(expression);
                }
            }
        }
    }

    fn finish_call(&mut self, callee: Expression) -> ParseExprResult {
        let mut arguments: Vec<Expression> = Vec::new();
        let callee = Box::new(callee);
        if self.next_matches_one(TokenType::CloseDelimiter(Delimiter::Paren)) {
            self.advance();
            return Ok(Expression::Call {
                callee,
                close_paren: self.current.unwrap().clone(),
                arguments,
            });
        }
        arguments.push(self.expression()?);
        while self.next_matches_one(TokenType::SingleChar(SingleCharTokenType::Comma)) {
            self.advance();
            if arguments.len() >= 255 {
                return Err(self.make_error("Can't have more than 255 arguments."));
            }
            arguments.push(self.expression()?);
        }
        self.advance_when_match(
            TokenType::CloseDelimiter(Delimiter::Paren),
            |parser| {
                Ok(Expression::Call {
                    callee,
                    close_paren: parser.current.unwrap().clone(),
                    arguments,
                })
            },
            |parser| Err(parser.make_error("Expect ')' after arguments.")),
        )
    }

    fn primary(&mut self) -> ParseExprResult {
        let next_token = *(self.tokens_iter.peek().unwrap());
        match &next_token.token_type {
            TokenType::Literal(literal) => {
                self.advance();
                Ok(literal.to_expression(next_token))
            }
            TokenType::Keyword(KeywordTokenType::Super) => {
                self.advance();
                self.find_super()
            }
            TokenType::Keyword(keyword) => {
                self.advance();
                Ok(keyword.to_expression(next_token).expect("Expect expression"))
            }
            TokenType::OpenDelimiter(delimiter) if *delimiter == Delimiter::Paren => {
                self.advance();
                self.find_group()
            }
            _ => Err(ParseError {
                token: next_token.clone(),
                message: "Expected expression",
            }),
        }
    }

    fn find_group(&mut self) -> ParseExprResult {
        let expression = self.expression()?;
        let close_paren_token = TokenType::CloseDelimiter(Delimiter::Paren);
        match self.tokens_iter.peek() {
            Some(token) if token.token_type == close_paren_token => {
                self.advance();
                Ok(Expression::Grouping(Box::new(expression)))
            }
            _ => Err(ParseError {
                token: (*self.tokens_iter.peek().unwrap()).clone(),
                message: "Expect ')' after expression.",
            }),
        }
    }

    fn find_super(&mut self) -> ParseExprResult {
        self.advance_when_match(
            TokenType::SingleChar(SingleCharTokenType::Dot),
            |parser| {
                parser.consume_identifier(|| "Expect superclass method name.")
                    .map(|method| Expression::Super {
                        keyword_token: parser.current.unwrap().clone(),
                        method
                    })
            },
            |parser| Err(parser.make_error("Expect '.' after 'super'."))
        )
    }

    fn find_binary_expression<EL, EF>(
        &mut self,
        expr_lookup: EL,
        expr_factory: EF,
        token_types: &[TokenType],
    ) -> ParseExprResult
    where
        EL: Fn(&mut Parser<'a>) -> ParseExprResult,
        EF: Fn(Box<Expression>, Token, Box<Expression>) -> Expression,
    {
        let mut expression = expr_lookup(self)?;
        while self.next_matches_any(token_types) {
            self.advance();
            let operator = self.current.unwrap();

            let right_expression = expr_lookup(self)?;
            expression = expr_factory(
                Box::new(expression),
                operator.clone(),
                Box::new(right_expression),
            )
        }
        Ok(expression)
    }

    fn next_matches_one(&mut self, token_type: TokenType) -> bool {
        if let Some(next) = self.tokens_iter.peek() {
            next.token_type == token_type
        } else {
            false
        }
    }

    fn next_matches_any(&mut self, token_types: &[TokenType]) -> bool {
        if let Some(next) = self.tokens_iter.peek() {
            token_types.iter().any(|v| v == &next.token_type)
        } else {
            false
        }
    }

    fn consume_identifier<F: FnOnce() -> &'static str>(
        &mut self,
        err_message: F,
    ) -> Result<String, ParseError> {
        match &self.tokens_iter.peek().unwrap().token_type {
            TokenType::Literal(LiteralTokenType::Identifier(name)) => {
                self.advance();
                Ok(name.clone())
            }
            _ => Err(self.make_error(err_message())),
        }
    }

    fn advance(&mut self) -> Option<&'a Token> {
        self.current = self.tokens_iter.next();
        self.current
    }

    fn advance_when_match<F, R, E>(&mut self, token_type: TokenType, next_step: F, else_fn: E) -> R
    where
        F: FnOnce(&mut Self) -> R,
        E: FnOnce(&Self) -> R,
    {
        if self.next_matches_one(token_type) {
            self.advance();
            next_step(self)
        } else {
            else_fn(self)
        }
    }

    fn make_error(&self, message: &'static str) -> ParseError {
        ParseError {
            token: self.current.unwrap().clone(),
            message,
        }
    }

    fn synchronize(&mut self) {
        let mut previous = self.current;
        self.advance();
        loop {
            if let Some(TokenType::SingleChar(SingleCharTokenType::Semicolon)) = previous.map(|v| &v.token_type) {
                return;
            }
            let token = self.tokens_iter.peek().unwrap();
            match &token.token_type {
                TokenType::Eof => return,
                TokenType::Keyword(keyword) => {
                    match keyword {
                        KeywordTokenType::Class
                        | KeywordTokenType::For
                        | KeywordTokenType::Fun
                        | KeywordTokenType::Var
                        | KeywordTokenType::If
                        | KeywordTokenType::While
                        | KeywordTokenType::Print
                        | KeywordTokenType::Return => return,
                        _ => {}
                    }
                }
                _ => {}
            }
            previous = self.current;
            self.advance();
        }
    }

    fn check_semicolon_after_stmt(&mut self, stmt: Statement) -> ParseStmtResult {
        if self.next_matches_one(TokenType::SingleChar(SingleCharTokenType::Semicolon)) {
            self.advance();
            Ok(stmt)
        } else {
            Err(self.make_error("Expect ';' after return value."))
        }
    }
}

impl LiteralTokenType {
    fn to_expression(&self, token: &Token) -> Expression {
        match self {
            LiteralTokenType::Identifier(name) => Expression::Variable(
                VariableExpression {
                    name: name.to_string(),
                    token: token.clone(),
                }
            ),
            LiteralTokenType::Number(number) => {
                Expression::Literal(LiteralExpression::Number(*number))
            }
            LiteralTokenType::String(string) => {
                Expression::Literal(LiteralExpression::String(string.clone()))
            }
        }
    }
}

impl KeywordTokenType {
    fn to_expression(&self, token: &Token) -> Option<Expression> {
        match self {
            KeywordTokenType::This => Some(Expression::This(token.clone())),
            KeywordTokenType::False => Some(Expression::Literal(LiteralExpression::False)),
            KeywordTokenType::True => Some(Expression::Literal(LiteralExpression::True)),
            KeywordTokenType::Nil => Some(Expression::Literal(LiteralExpression::Nil)),
            _ => None,
        }
    }
}

struct ParseError {
    token: Token,
    message: &'static str,
}

impl Error for ParseError {
    fn message(&self) -> &str {
        self.message
    }

    fn line(&self) -> usize {
        self.token.line as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_that_parser_generates_correct_output_for_binary_expression() {
        let plus_token = Token::new(
            TokenType::SingleChar(SingleCharTokenType::Plus),
            String::from("+"),
            1,
            1
        );
        let tokens = vec![
            Token::new(
                TokenType::Literal(LiteralTokenType::Number(123f64)),
                String::from("123"),
                1,
                2
            ),
            plus_token.clone(),
            Token::new(
                TokenType::Literal(LiteralTokenType::Number(123f64)),
                String::from("123"),
                1,
                3
            ),
            Token::new(TokenType::Eof, String::new(), 1, 4),
        ];
        let mut parser = Parser::new(&tokens);
        let statement = parser.parse();
        let expected_expression = Expression::Binary(
            Box::new(Expression::Literal(LiteralExpression::Number(123f64))),
            plus_token.clone(),
            Box::new(Expression::Literal(LiteralExpression::Number(123f64))),
        );
        assert_eq!(statement, vec![Statement::Expression(expected_expression)])
    }
}
