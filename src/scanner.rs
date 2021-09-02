use crate::token::Object;
use crate::token::Token;
use crate::token::TokenType;
use crate::LoxError;
use std::collections::HashMap;
use std::str::FromStr;

type Result<T> = crate::Result<T>;

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,

    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source,
            tokens: vec![],

            start: 0usize,
            current: 0usize,
            line: 1usize,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>> {
        let mut errs = vec![];
        while !self.is_at_end() {
            self.start = self.current;
            if let Err(mut err) = self.scan_token() {
                errs.append(&mut err);
            }
        }

        self.tokens
            .push(Token::new(TokenType::Eof, "".to_string(), None, self.line));

        if errs.is_empty() {
            Ok(self.tokens.clone())
        } else {
            Err(errs)
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.chars().count()
    }

    fn scan_token(&mut self) -> Result<()> {
        let c: char = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                if self.matches('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }
            '=' => {
                if self.matches('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }
            '<' => {
                if self.matches('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            }
            '>' => {
                if self.matches('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => {
                self.line += 1;
            }
            '"' => return self.string(),
            c => {
                if c.is_numeric() {
                    self.number();
                } else if c.is_alphabetic() || c == '_' {
                    self.identifier();
                } else {
                    return LoxError::error(self.line, format!("Unexpected character {}", c));
                }
            }
        }
        Ok(())
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text: String = self
            .source
            .chars()
            .skip(self.start)
            .take(self.current - self.start)
            .collect();
        let keywords: HashMap<&str, TokenType> = [
            ("and", TokenType::And),
            ("class", TokenType::Class),
            ("else", TokenType::Else),
            ("false", TokenType::False),
            ("fun", TokenType::Fun),
            ("for", TokenType::For),
            ("if", TokenType::If),
            ("nil", TokenType::Nil),
            ("or", TokenType::Or),
            ("print", TokenType::Print),
            ("return", TokenType::Return),
            ("super", TokenType::Super),
            ("this", TokenType::This),
            ("true", TokenType::True),
            ("var", TokenType::Var),
            ("while", TokenType::While),
        ]
        .iter()
        .cloned()
        .collect();
        if let Some(kind) = keywords.get(&*text) {
            self.add_token(*kind)
        } else {
            self.add_token(TokenType::Identifier)
        }
    }

    fn number(&mut self) {
        while self.peek().is_numeric() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_numeric() {
            self.advance();

            while self.peek().is_numeric() {
                self.advance();
            }
        }

        let text: String = self
            .source
            .chars()
            .skip(self.start)
            .take(self.current - self.start)
            .collect();
        self.add_token_literal(
            TokenType::Number,
            Some(Object::Number(f64::from_str(&text).unwrap())),
        )
    }

    fn string(&mut self) -> Result<()> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return LoxError::error(self.line, "Unterminated string.".to_string());
        }

        self.advance();

        let text = self
            .source
            .chars()
            .skip(self.start + 1)
            .take(self.current - self.start - 2)
            .collect();

        self.add_token_literal(TokenType::String, Some(Object::String(text)));
        Ok(())
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap()
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.chars().count() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap()
        }
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current);
        self.current += 1;
        c.unwrap()
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source.chars().nth(self.current).unwrap() != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn add_token(&mut self, kind: TokenType) {
        self.add_token_literal(kind, None)
    }

    fn add_token_literal(&mut self, kind: TokenType, value: Option<Object>) {
        let text = self
            .source
            .chars()
            .skip(self.start)
            .take(self.current - self.start)
            .collect();

        self.tokens.push(Token::new(kind, text, value, self.line));
    }
}
