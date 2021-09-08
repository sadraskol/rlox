use crate::expr::Expr;
use crate::expr::Stmt;
use crate::token::Object;
use crate::token::Token;
use crate::token::TokenType;

use crate::LoxError;

type Result<T> = crate::Result<T>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut vec = vec![];
        for token in tokens {
            vec.push(token);
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
        } else if self.matches(&[TokenType::Fun]) {
            self.function()
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

        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after variable declaration".to_string(),
        )?;
        Ok(Stmt::Var(name, initializer))
    }

    fn function(&mut self) -> Result<Stmt> {
        let name = self.consume(&TokenType::Identifier, "Expect function name.".to_string())?;
        self.consume(
            &TokenType::LeftParen,
            "Expect '(' after function name.".to_string(),
        )?;
        let mut parameters: Vec<Token> = vec![];
        if !self.check(&TokenType::RightParen) {
            parameters
                .push(self.consume(&TokenType::Identifier, "Expect parameter name".to_string())?);
            while self.matches(&[TokenType::Comma]) {
                if parameters.len() >= u16::MAX as usize {
                    self.error(
                        self.peek(),
                        format!("Can't have more than {} arguments", u16::MAX),
                    )?;
                }
                parameters.push(
                    self.consume(&TokenType::Identifier, "Expect parameter name".to_string())?,
                );
            }
        }
        self.consume(
            &TokenType::RightParen,
            "Expect ')' after parameters".to_string(),
        )?;
        self.consume(
            &TokenType::LeftBrace,
            "Expect '{' before function body.".to_string(),
        )?;
        let body = self.block()?;
        Ok(Stmt::Fn(name, parameters, body))
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.matches(&[TokenType::Print]) {
            self.print_statement()
        } else if self.matches(&[TokenType::Return]) {
            self.returns()
        } else if self.matches(&[TokenType::LeftBrace]) {
            Ok(Stmt::Block(self.block()?))
        } else if self.matches(&[TokenType::If]) {
            self.if_statement()
        } else if self.matches(&[TokenType::While]) {
            self.while_statement()
        } else if self.matches(&[TokenType::For]) {
            self.for_statement()
        } else {
            self.expr_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value.".to_string())?;
        Ok(Stmt::Print(expr))
    }

    fn returns(&mut self) -> Result<Stmt> {
        let token = self.previous();
        let expr = if !self.check(&TokenType::Semicolon) {
            self.expression()?
        } else {
            Expr::Literal(Object::Nil)
        };

        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after return value.".to_string(),
        )?;
        Ok(Stmt::Return(token, expr))
    }

    fn expr_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value.".to_string())?;
        Ok(Stmt::Expr(expr))
    }

    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = vec![];
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(
            &TokenType::RightBrace,
            "Expect '}' after block.".to_string(),
        )?;
        Ok(statements)
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after if.".to_string())?;
        let expr = self.expression()?;
        self.consume(
            &TokenType::RightParen,
            "Expect ')' after if condition.".to_string(),
        )?;
        let then_statement = self.statement()?;

        let mut else_statement = None;
        if self.matches(&[TokenType::Else]) {
            else_statement = Some(Box::new(self.statement()?));
        }
        Ok(Stmt::If(expr, Box::new(then_statement), else_statement))
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after while.".to_string())?;
        let expr = self.expression()?;
        self.consume(
            &TokenType::RightParen,
            "Expect ')' after while condition.".to_string(),
        )?;

        let body = self.statement()?;

        Ok(Stmt::While(expr, Box::new(body)))
    }

    fn for_statement(&mut self) -> Result<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after for.".to_string())?;
        let initializer = if self.matches(&[TokenType::Semicolon]) {
            None
        } else if self.matches(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expr_statement()?)
        };

        let cond = if self.check(&TokenType::Semicolon) {
            Expr::Literal(Object::Bool(true))
        } else {
            self.expression()?
        };
        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after loop condition.".to_string(),
        )?;

        let increment = if !self.check(&TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            &TokenType::RightParen,
            "Expect ')' after for clauses.".to_string(),
        )?;

        let mut body = self.statement()?;

        body = if let Some(inc) = increment {
            Stmt::Block(vec![body, Stmt::Expr(inc)])
        } else {
            body
        };

        body = Stmt::While(cond, Box::new(body));

        Ok(if let Some(init) = initializer {
            Stmt::Block(vec![init, body])
        } else {
            body
        })
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or()?;

        if self.matches(&[TokenType::Equal]) {
            let token = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                Ok(Expr::Assign(name, Box::new(value)))
            } else {
                self.error(token, "Invalid assignment target.".to_string())
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Expr> {
        let mut expr = self.and()?;

        while self.matches(&[TokenType::Or]) {
            let op = self.previous();
            let right = self.and()?;
            expr = Expr::Logical(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;

        while self.matches(&[TokenType::And]) {
            let op = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        while self.matches(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let op = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;

        while self.matches(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = self.previous();
            let right = self.term()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while self.matches(&[TokenType::Minus, TokenType::Plus]) {
            let op = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while self.matches(&[TokenType::Slash, TokenType::Star]) {
            let op = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.matches(&[TokenType::Minus, TokenType::Bang]) {
            let op = self.previous();
            let right = self.unary()?;
            Ok(Expr::Unary(op, Box::new(right)))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.matches(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut args = vec![];
        if !self.check(&TokenType::RightParen) {
            args.push(self.expression()?);
            while self.matches(&[TokenType::Comma]) {
                if args.len() > u16::MAX as usize {
                    self.error(
                        self.peek(),
                        format!("Can't have more that {} arguments.", u16::MAX),
                    )?;
                }
                args.push(self.expression()?);
            }
        }

        let token = self.consume(
            &TokenType::RightParen,
            "Expect ')' after arguments.".to_string(),
        )?;

        Ok(Expr::Call(Box::new(callee), token, args))
    }

    fn primary(&mut self) -> Result<Expr> {
        if self.matches(&[TokenType::False]) {
            Ok(Expr::Literal(Object::Bool(false)))
        } else if self.matches(&[TokenType::True]) {
            Ok(Expr::Literal(Object::Bool(true)))
        } else if self.matches(&[TokenType::Identifier]) {
            Ok(Expr::Variable(self.previous()))
        } else if self.matches(&[TokenType::Nil]) {
            Ok(Expr::Literal(Object::Nil))
        } else if self.matches(&[TokenType::Number, TokenType::String]) {
            let lit = self.previous().literal.as_ref().unwrap().clone();
            Ok(Expr::Literal(lit))
        } else if self.matches(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(
                &TokenType::RightParen,
                "Expect ')' after expression.".to_string(),
            )?;
            Ok(Expr::Grouping(Box::new(expr)))
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

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenType::Eof
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.current).unwrap().clone()
    }

    fn previous(&self) -> Token {
        self.tokens.get(self.current - 1).unwrap().clone()
    }

    fn consume(&mut self, kind: &TokenType, message: String) -> Result<Token> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            self.error(self.peek(), message)
        }
    }

    fn error<Any>(&mut self, token: Token, message: String) -> Result<Any> {
        Err(LoxError::error_tok(&token, message))
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
