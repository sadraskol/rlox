use crate::expr::Expr;
use crate::token::Object;
use crate::token::Token;
use crate::token::TokenType;

use crate::LoxError;

type Result<T> = crate::Result<T>;

pub struct Parser {
    tokens: Vec<Box<Token>>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut vec = vec![];
        for token in tokens {
            vec.push(Box::new(token));
        }
        Parser {
            tokens: vec,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Box<Expr>> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Box<Expr>> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.comparison()?;

        while self.matches(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let op = self.previous();
            let right = self.comparison()?;
            expr = Box::new(Expr::Binary(expr, op, right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.term()?;

        while self.matches(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = self.previous();
            let right = self.term()?;
            expr = Box::new(Expr::Binary(expr, op, right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.factor()?;

        while self.matches(&[TokenType::Minus, TokenType::Plus]) {
            let op = self.previous();
            let right = self.factor()?;
            expr = Box::new(Expr::Binary(expr, op, right));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.unary()?;

        while self.matches(&[TokenType::Slash, TokenType::Star]) {
            let op = self.previous();
            let right = self.unary()?;
            expr = Box::new(Expr::Binary(expr, op, right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Box<Expr>> {
        if self.matches(&[TokenType::Minus, TokenType::Bang]) {
            let op = self.previous();
            let right = self.unary()?;
            Ok(Box::new(Expr::Unary(op, right)))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Box<Expr>> {
        if self.matches(&[TokenType::False]) {
            Ok(Box::new(Expr::Literal(Object::Bool(false))))
        } else if self.matches(&[TokenType::True]) {
            Ok(Box::new(Expr::Literal(Object::Bool(true))))
        } else if self.matches(&[TokenType::Nil]) {
            Ok(Box::new(Expr::Literal(Object::Nil)))
        } else if self.matches(&[TokenType::Number, TokenType::String]) {
            let lit = self.previous().as_ref().literal.as_ref().unwrap().clone();
            Ok(Box::new(Expr::Literal(lit)))
        } else if self.matches(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(
                &TokenType::RightParen,
                "Expect ')' after expression.".to_string(),
            )?;
            Ok(Box::new(Expr::Grouping(expr)))
        } else {
            self.error(self.peek(), "Expect expression.".to_string())
        }
    }

    fn matches(&mut self, types: &[TokenType]) -> bool {
        for kind in types {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, kind: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().kind == kind
        }
    }

    fn advance(&mut self) -> Box<Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenType::Eof
    }

    fn peek(&self) -> Box<Token> {
        self.tokens.get(self.current).unwrap().clone()
    }

    fn previous(&self) -> Box<Token> {
        self.tokens.get(self.current - 1).unwrap().clone()
    }

    fn consume(&mut self, kind: &TokenType, message: String) -> Result<Box<Token>> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            self.error(self.peek(), message)
        }
    }

    fn error<Any>(&mut self, token: Box<Token>, message: String) -> Result<Any> {
        LoxError::error_tok(token, message)
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().kind == TokenType::Semicolon {
                return;
            }

            match self.peek().kind {
                TokenType::Class
                | TokenType::For
                | TokenType::Fun
                | TokenType::If
                | TokenType::Print
                | TokenType::Return
                | TokenType::Var
                | TokenType::While => {
                    return;
                }
                _ => self.advance(),
            };
        }
    }
}
