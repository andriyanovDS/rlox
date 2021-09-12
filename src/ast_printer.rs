use crate::expression::{Expression, LiteralExpression, Visitor};
use crate::token::Token;

struct AstPrinter;

impl Visitor<String> for AstPrinter {
    fn visit_binary(&self, left: &Expression, operator: &Token, right: &Expression) -> String {
        self.parenthize(operator.lexeme.iter().collect(), vec![left, right])
    }

    fn visit_grouping(&self, expression: &Expression) -> String {
        self.parenthize(String::from("group"), vec![expression])
    }

    fn visit_literal(&self, literal: &LiteralExpression) -> String {
        match literal {
            LiteralExpression::True => String::from("true"),
            LiteralExpression::False => String::from("false"),
            LiteralExpression::Nil => String::from("true"),
            LiteralExpression::String(string) => string.clone(),
            LiteralExpression::Number(number) => number.to_string(),
        }
    }

    fn visit_unary(&self, operator: &Token, right: &Expression) -> String {
        self.parenthize(operator.lexeme.iter().collect(), vec![right])
    }

    fn visit_variable(&self, literal: String) -> String {
        literal
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
    use crate::token_type::{SingleCharTokenType, TokenType};

    #[test]
    fn it_works() {
        let left_expression = Expression::Unary(
            Token::new(
                TokenType::SingleChar(SingleCharTokenType::Minus),
                vec!['-'],
                0,
            ),
            Box::new(Expression::Literal(LiteralExpression::Number(12f64))),
        );
        let right_expression = Expression::Grouping(Box::new(Expression::Literal(
            LiteralExpression::Number(45.67f64),
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
