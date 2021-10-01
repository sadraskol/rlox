struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    fn init(source: &'a str) -> Self {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn compile(&mut self) {
        let mut line = 0;
        loop {
            let token = self.scan_token();
            if token.line != line {
                print!("{:4} ", token.line);
                line = token.line;
            } else {
                print!("   | ");
            }
            println!("{:?} '{}'", token.kind, token.lexeme);

            if token.kind == TokenType::Eof {
                break;
            }
        }
    }

    fn scan_token(&mut self) -> Token<'_> {
        self.skip_whitespace();
        self.start = self.current;
        if self.is_at_end() {
            self.make_token(TokenType::Eof)
        } else {
            let c = self.advance();

            if c.is_alphabetic() || c == '_' {
                return self.identifier();
            }
            if c.is_numeric() {
                return self.number();
            }

            match c {
                '(' => self.make_token(TokenType::LeftParen),
                ')' => self.make_token(TokenType::RightParen),
                '{' => self.make_token(TokenType::LeftBrace),
                '}' => self.make_token(TokenType::RightBrace),
                ';' => self.make_token(TokenType::Semicolon),
                ',' => self.make_token(TokenType::Comma),
                '.' => self.make_token(TokenType::Dot),
                '-' => self.make_token(TokenType::Minus),
                '+' => self.make_token(TokenType::Plus),
                '/' => self.make_token(TokenType::Slash),
                '*' => self.make_token(TokenType::Star),
                '!' => {
                    if self.matches('=') {
                        self.make_token(TokenType::BangEqual)
                    } else {
                        self.make_token(TokenType::Bang)
                    }
                }
                '=' => {
                    if self.matches('=') {
                        self.make_token(TokenType::EqualEqual)
                    } else {
                        self.make_token(TokenType::Equal)
                    }
                }
                '<' => {
                    if self.matches('=') {
                        self.make_token(TokenType::LessEqual)
                    } else {
                        self.make_token(TokenType::Less)
                    }
                }
                '>' => {
                    if self.matches('=') {
                        self.make_token(TokenType::GreaterEqual)
                    } else {
                        self.make_token(TokenType::Greater)
                    }
                }
                '"' => self.string(),
                _ => self.error_token("Unexpected character."),
            }
        }
    }

    fn identifier(&mut self) -> Token<'_> {
        while self.peek().is_alphabetic() || self.peek() == '_' || self.peek().is_numeric() {
            self.advance();
        }
        self.make_token(self.identifier_type())
    }

    fn identifier_type(&self) -> TokenType {
        match self.source.chars().nth(self.start).unwrap() {
            'a' => self.check_keyword(1, 2, "nd", TokenType::And),
            'c' => self.check_keyword(1, 4, "lass", TokenType::Class),
            'e' => self.check_keyword(1, 3, "lse", TokenType::Else),
            'f' => {
                if self.current - self.start > 1 {
                    match self.source.chars().nth(self.start + 1).unwrap() {
                        'a' => self.check_keyword(2, 3, "lse", TokenType::False),
                        'o' => self.check_keyword(2, 1, "r", TokenType::For),
                        'u' => self.check_keyword(2, 1, "n", TokenType::Fun),
                        _ => TokenType::Identifier
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'i' => self.check_keyword(1, 1, "f", TokenType::If),
            'n' => self.check_keyword(1, 2, "il", TokenType::Nil),
            'o' => self.check_keyword(1, 1, "r", TokenType::Or),
            'p' => self.check_keyword(1, 4, "rint", TokenType::Print),
            'r' => self.check_keyword(1, 5, "eturn", TokenType::Return),
            's' => self.check_keyword(1, 4, "uper", TokenType::Super),
            't' => {
                if self.current - self.start > 1 {
                    match self.source.chars().nth(self.start + 1).unwrap() {
                        'h' => self.check_keyword(2, 2, "is", TokenType::This),
                        'r' => self.check_keyword(2, 2, "ue", TokenType::True),
                        _ => TokenType::Identifier
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'v' => self.check_keyword(1, 2, "ar", TokenType::Var),
            'w' => self.check_keyword(1, 4, "hile", TokenType::While),
            _ => TokenType::Identifier
        }
    }

    fn check_keyword(&self, start: usize, length: usize, rest: &str, kind: TokenType) -> TokenType {
        if self.current - self.start == start + length && rest == &self.source[self.start + start..self.start + start + length] {
            kind
        } else {
            TokenType::Identifier
        }
    }
 
    fn number(&mut self) -> Token<'_> {
        while self.peek().is_numeric() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().map(|c| c.is_numeric()).unwrap_or(false) {
            self.advance();

            while self.peek().is_numeric() {
                self.advance();
            }
        }
        self.make_token(TokenType::Number)
    }

    fn string(&mut self) -> Token<'_> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.error_token("Unterminated string.")
        } else {
            self.advance();
            self.make_token(TokenType::String)
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            if c.is_whitespace() {
                if c == '\n' {
                    self.line += 1;
                }
                self.advance();
            } else if c == '/' {
                if self.peek_next() == Some('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    return;
                }
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> char {
        self.source.chars().nth(self.current).unwrap()
    }

    fn peek_next(&self) -> Option<char> {
        self.source.chars().nth(self.current + 1)
    }

    fn matches(&mut self, c: char) -> bool {
        if self.source.chars().nth(self.current) == Some(c) {
            self.current += 1;
            true
        } else {
            false
        }
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current - 1).unwrap()
    }

    fn is_at_end(&self) -> bool {
        self.current == self.source.chars().count()
    }

    fn make_token(&self, kind: TokenType) -> Token<'_> {
        Token {
            kind,
            lexeme: &self.source[self.start..self.current],
            line: self.line,
        }
    }

    fn error_token(&self, msg: &'a str) -> Token<'a> {
        Token {
            kind: TokenType::Error,
            lexeme: msg,
            line: self.line,
        }
    }
}

struct Token<'a> {
    kind: TokenType,
    lexeme: &'a str,
    line: usize,
}

#[derive(Debug, PartialEq, Eq)]
enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Error,
    Eof,
}
