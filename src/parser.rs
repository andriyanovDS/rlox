use crate::expression::{Expression, LiteralExpression};
use crate::statement::Statement;
use crate::token::Token;
use crate::token_type::TokenType::EOF;
use crate::token_type::{
    Delimiter, ExpressionOperatorTokenType, KeywordTokenType, LiteralTokenType,
    SingleCharTokenType, TokenType,
};
use std::iter::Peekable;
use std::slice::Iter;

pub struct Parser<'a> {
    tokens_iter: Peekable<Iter<'a, Token>>,
    current: Option<&'a Token>,
    previous: Option<&'a Token>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens_iter: tokens.iter().peekable(),
            current: None,
            previous: None,
        }
    }

    pub fn parse(&mut self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = Vec::new();
        loop {
            match self.statement() {
                Ok(expression) => {
                    statements.push(expression);
                }
                Err(error) if error.token.token_type == TokenType::EOF => break,
                Err(error) => {
                    self.synchronize();
                    eprint!("{}", error.error_message());
                }
            }
        }
        statements
    }

    fn declaration(&mut self) -> Result<Statement, ParseError> {
        let var_token_type = TokenType::Keyword(KeywordTokenType::Var);
        if self.tokens_iter.peek().unwrap().token_type == var_token_type {
            self.variable_statement()
        } else {
            self.statement()
        }
    }

    fn statement(&mut self) -> Result<Statement, ParseError> {
        match self.tokens_iter.peek().unwrap().token_type {
            TokenType::Keyword(KeywordTokenType::Var) => {
                self.advance();
                self.variable_statement()
            }
            TokenType::Keyword(KeywordTokenType::Print) => {
                self.advance();
                self.print_statement()
            }
            _ => self.expression_statement(),
        }
    }

    fn variable_statement(&mut self) -> Result<Statement, ParseError> {
        let token = self.tokens_iter.peek().unwrap();
        if let TokenType::Literal(LiteralTokenType::Identifier(ref name)) = token.token_type {
            self.advance();
            self.make_variable_stmt(name.to_string())
                .and_then(|stmt| self.check_semicolon_after_stmt(stmt))
        } else {
            Err(ParseError {
                token: (*token).clone(),
                message: "Expect variable name.",
            })
        }
    }

    fn make_variable_stmt(&mut self, name: String) -> Result<Statement, ParseError> {
        let equal_type = TokenType::ExpressionOperator(ExpressionOperatorTokenType::Equal);
        if self.next_matches_one(&equal_type) {
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

    fn print_statement(&mut self) -> Result<Statement, ParseError> {
        self.expression()
            .map(Statement::Print)
            .and_then(|stmt| self.check_semicolon_after_stmt(stmt))
    }

    fn expression_statement(&mut self) -> Result<Statement, ParseError> {
        self.expression()
            .map(Statement::Expression)
            .and_then(|stmt| self.check_semicolon_after_stmt(stmt))
    }

    fn expression(&mut self) -> Result<Expression, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expression, ParseError> {
        let token_types = vec![
            TokenType::ExpressionOperator(ExpressionOperatorTokenType::NotEqual),
            TokenType::ExpressionOperator(ExpressionOperatorTokenType::EqualEqual),
        ];
        self.find_binary_expression(Parser::comparison, &token_types)
    }

    fn comparison(&mut self) -> Result<Expression, ParseError> {
        let token_types = vec![
            TokenType::ExpressionOperator(ExpressionOperatorTokenType::Greater),
            TokenType::ExpressionOperator(ExpressionOperatorTokenType::GreaterEqual),
            TokenType::ExpressionOperator(ExpressionOperatorTokenType::Less),
            TokenType::ExpressionOperator(ExpressionOperatorTokenType::LessEqual),
        ];
        self.find_binary_expression(Parser::term, &token_types)
    }

    fn term(&mut self) -> Result<Expression, ParseError> {
        let token_types = vec![
            TokenType::SingleChar(SingleCharTokenType::Minus),
            TokenType::SingleChar(SingleCharTokenType::Plus),
        ];
        self.find_binary_expression(Parser::factor, &token_types)
    }

    fn factor(&mut self) -> Result<Expression, ParseError> {
        let token_types = vec![
            TokenType::SingleChar(SingleCharTokenType::Slash),
            TokenType::SingleChar(SingleCharTokenType::Star),
        ];
        self.find_binary_expression(Parser::unary, &token_types)
    }

    fn unary(&mut self) -> Result<Expression, ParseError> {
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
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expression, ParseError> {
        let next_token = self.tokens_iter.peek().unwrap().clone();
        match &next_token.token_type {
            TokenType::Literal(literal) => {
                self.advance();
                Ok(literal.to_expression(next_token))
            }
            TokenType::Keyword(keyword) => {
                self.advance();
                Ok(keyword.to_expression().expect("Expect expression"))
            }
            TokenType::OpenDelimiter(delimiter) if *delimiter == Delimiter::Paren => {
                self.advance();
                self.find_group()
            }
            _ => Err(ParseError {
                token: (*next_token).clone(),
                message: "Expected expression",
            }),
        }
    }

    fn find_group(&mut self) -> Result<Expression, ParseError> {
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

    fn find_binary_expression<F: Fn(&mut Parser<'a>) -> Result<Expression, ParseError>>(
        &mut self,
        expression_factory: F,
        token_types: &[TokenType],
    ) -> Result<Expression, ParseError> {
        let mut expression = expression_factory(self)?;

        while self.next_matches_any(token_types) {
            self.advance();
            let operator = self.current.unwrap();

            let right_expression = expression_factory(self)?;
            expression = Expression::Binary(
                Box::new(expression),
                operator.clone(),
                Box::new(right_expression),
            )
        }
        Ok(expression)
    }

    fn next_matches_one(&mut self, token_type: &TokenType) -> bool {
        if let Some(next) = self.tokens_iter.peek() {
            &next.token_type == token_type
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

    fn advance(&mut self) -> Option<&'a Token> {
        self.previous = self.current.take();
        self.current = self.tokens_iter.next();
        self.current
    }

    fn synchronize(&mut self) {
        if let Some(token) = self.current {
            if TokenType::SingleChar(SingleCharTokenType::Semicolon) == token.token_type {
                return;
            }
        }
        while let Some(token) = self.advance() {
            if let TokenType::Keyword(ref keyword) = token.token_type {
                match keyword {
                    KeywordTokenType::Class
                    | KeywordTokenType::For
                    | KeywordTokenType::Fun
                    | KeywordTokenType::Var
                    | KeywordTokenType::If
                    | KeywordTokenType::While
                    | KeywordTokenType::Print
                    | KeywordTokenType::Return => return,
                    _ => continue,
                }
            }
        }
    }

    fn check_semicolon_after_stmt(&mut self, stmt: Statement) -> Result<Statement, ParseError> {
        if self.next_matches_one(&TokenType::SingleChar(SingleCharTokenType::Semicolon)) {
            self.advance();
            Ok(stmt)
        } else {
            Err(ParseError {
                token: self.current.unwrap().clone(),
                message: "Expect ';' after return value.",
            })
        }
    }
}

impl LiteralTokenType {
    fn to_expression(&self, token: &Token) -> Expression {
        match self {
            LiteralTokenType::Identifier(name) => Expression::Variable {
                name: name.to_string(),
                token: token.clone(),
            },
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
    fn to_expression(&self) -> Option<Expression> {
        match self {
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

impl ParseError {
    fn error_message(&self) -> String {
        let lexeme: String = self.token.lexeme.iter().collect();
        format!("{} at '{}' {}", self.token.line, lexeme, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::Object;

    #[test]
    fn test_that_parser_generates_correct_output_for_binary_expression() {
        let plus_token = Token::new(
            TokenType::SingleChar(SingleCharTokenType::Plus),
            vec!['+'],
            1,
        );
        let tokens = vec![
            Token::new(
                TokenType::Literal(LiteralTokenType::Number(123f64)),
                vec!['1', '2', '3'],
                1,
            ),
            plus_token.clone(),
            Token::new(
                TokenType::Literal(LiteralTokenType::Number(123f64)),
                vec!['1', '2', '3'],
                1,
            ),
            Token::new(TokenType::EOF, vec![], 1),
        ];
        let mut parser = Parser::new(&tokens);
        let expressions = parser.parse();
        let expected_expression = Expression::Binary(
            Box::new(Expression::Literal(LiteralExpression::Number(123f64))),
            plus_token.clone(),
            Box::new(Expression::Literal(LiteralExpression::Number(123f64))),
        );
        assert_eq!(expressions, vec![expected_expression])
    }
}
