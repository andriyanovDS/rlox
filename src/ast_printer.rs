use crate::expression::{Expression, Visitor};
use crate::token::Token;
use crate::token_type::{LiteralTokenType, SingleCharTokenType, TokenType};

struct AstPrinter;

impl Visitor<String> for AstPrinter {
    fn visit_binary(&self, left: &Expression, operator: &Token, right: &Expression) -> String {
        self.parenthize(operator.lexeme.iter().collect(), vec![left, right])
    }

    fn visit_grouping(&self, expression: &Expression) -> String {
        self.parenthize(String::from("group"), vec![expression])
    }

    fn visit_literal(&self, literal: &LiteralTokenType) -> String {
        match literal {
            LiteralTokenType::Identifier(id) => id.clone(),
            LiteralTokenType::String(string) => string.clone(),
            LiteralTokenType::Number(number) => number.to_string(),
        }
    }

    fn visit_unary(&self, operator: &Token, right: &Expression) -> String {
        self.parenthize(operator.lexeme.iter().collect(), vec![right])
    }
}

impl AstPrinter {
    fn parenthize(&self, name: String, expressions: Vec<&Expression>) -> String {
        let tokens: Vec<String> = expressions.iter().map(|v| v.accept(self)).collect();

        format!("({} {})", name, tokens.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let left_expression = Expression::Unary(
            Token::new(
                TokenType::SingleChar(SingleCharTokenType::Minus),
                vec!['-'],
                0,
            ),
            Box::new(Expression::Literal(LiteralTokenType::Number(123f32))),
        );
        let right_expression = Expression::Grouping(Box::new(Expression::Literal(
            LiteralTokenType::Number(45.67f32),
        )));
        let expression = Expression::Binary(
            Box::new(left_expression),
            Token::new(
                TokenType::SingleChar(SingleCharTokenType::Star),
                vec!['*'],
                1,
            ),
            Box::new(right_expression),
        );
        let ast_printer = AstPrinter {};
        let result = expression.accept(&ast_printer);

        assert_eq!(result, String::from("(* (- 123) (group 45.67))"));
    }
}
