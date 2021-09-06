use crate::expr::Expr;
use crate::expr::Stmt;
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

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        self.program()
    }

    fn program(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = vec![];
        while !self.is_at_end() {
            stmts.push(self.declaration()?);
        }

        Ok(stmts)
    }

    fn declaration(&mut self) -> Result<Stmt> {
        let res = if self.matches(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };
        if res.is_err() {
            self.synchronize();
            panic!("super");
        } else {
            res
        }
    }
    
    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(&TokenType::Identifier, "Expect variable name.".to_string())?;
        let mut initializer = None;
        if self.matches(&[TokenType::Equal]) {
            initializer = Some(self.expression()?);
        }

        self.consume(&TokenType::Semicolon, "Expect ';' after variable declaration".to_string())?;
        Ok(Stmt::Var(name, initializer))
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.matches(&[TokenType::Print]) {
            self.print_statement()
        } else if self.matches(&[TokenType::LeftBrace]) {
            self.block()
        } else if self.matches(&[TokenType::If]) {
            self.if_statement()
        } else {
            self.expr_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value.".to_string())?;
        Ok(Stmt::Print(expr))
    }

    fn expr_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value.".to_string())?;
        Ok(Stmt::Expr(expr))
    }

    fn block(&mut self) -> Result<Stmt> {
        let mut statements = vec![];
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(&TokenType::RightBrace, "Expect '}' after block.".to_string())?;
        Ok(Stmt::Block(statements))
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after if.".to_string())?;
        let expr = self.expression()?;
        self.consume(&TokenType::RightParen, "Expect ')' after if condition.".to_string())?;
        let then_statement = self.statement()?;

        let mut else_statement = None;
        if self.matches(&[TokenType::Else]) {
            else_statement = Some(Box::new(self.statement()?));
        }
        Ok(Stmt::If(expr, Box::new(then_statement), else_statement))
    }

    fn expression(&mut self) -> Result<Box<Expr>> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Box<Expr>> {
        let expr = self.equality()?;

        if self.matches(&[TokenType::Equal]) {
            let token = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable(name) = *expr {
                return Ok(Box::new(Expr::Assign(name, value)));
            } else {
                self.error(token, "Invalid assignment target.".to_string())
            }
        } else {
            Ok(expr)
        }

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
        } else if self.matches(&[TokenType::Identifier]) {
            Ok(Box::new(Expr::Variable(self.previous())))
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
        Err(LoxError::error_tok(token, message))
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
