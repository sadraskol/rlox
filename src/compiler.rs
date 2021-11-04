use crate::chunk::Chunk;
use crate::chunk::OpCode;
use crate::chunk::Value;
use std::str::FromStr;

#[derive(Clone, Copy, Debug)]
struct Local<'a> {
    token: Token<'a>,
    depth: Option<usize>,
}

struct Compiler<'a> {
    locals: Vec<Local<'a>>,
    scope_depth: usize,
}

impl<'a> Compiler<'a> {
    fn new() -> Self {
        Compiler {
            locals: vec![],
            scope_depth: 0,
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth = self.scope_depth + 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth = self.scope_depth - 1;
    }

    fn variable_already_declared(&self, token: &Token<'a>) -> bool {
        for local in self.locals.iter().rev() {
            if local.token.lexeme == token.lexeme && local.depth == Some(self.scope_depth) {
                return true;
            }
        }
        false
    }

    fn add_local(&mut self, token: Token<'a>) {
        self.locals.push(Local { token, depth: None })
    }

    fn locals_removed_from_stack(&mut self) -> usize {
        let mut locals_off_the_stack = 0;
        let mut new_locals = vec![];
        for l in self.locals.drain(..) {
            if let Some(d) = l.depth {
                if d <= self.scope_depth {
                    new_locals.push(l);
                } else {
                    locals_off_the_stack += 1;
                }
            } else {
                new_locals.push(l);
            }
        }
        self.locals = new_locals;
        locals_off_the_stack
    }
}

pub struct Parser<'a> {
    scanner: Scanner<'a>,
    compiler: Compiler<'a>,
    previous: Token<'a>,
    current: Token<'a>,
    chunk: Option<Chunk>,
    had_error: bool,
    panic_mode: bool,
}

enum Prefix {
    None,
    Variable,
    Grouping,
    Unary,
    Number,
    Literal,
    String,
}

enum Infix {
    None,
    Binary,
    Or,
    And,
}

struct Rule {
    prefix: Prefix,
    infix: Infix,
    precedence: Precedence,
}

impl Rule {
    fn init(prefix: Prefix, infix: Infix, precedence: Precedence) -> Self {
        Rule {
            prefix,
            infix,
            precedence,
        }
    }
}

fn get_rule(kind: &TokenType) -> Rule {
    match kind {
        TokenType::LeftParen => Rule::init(Prefix::Grouping, Infix::None, Precedence::None),
        TokenType::RightParen => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::LeftBrace => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::RightBrace => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::Comma => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::Dot => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::Minus => Rule::init(Prefix::Unary, Infix::Binary, Precedence::Term),
        TokenType::Plus => Rule::init(Prefix::None, Infix::Binary, Precedence::Term),
        TokenType::Semicolon => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::Slash => Rule::init(Prefix::None, Infix::Binary, Precedence::Factor),
        TokenType::Star => Rule::init(Prefix::None, Infix::Binary, Precedence::Factor),
        TokenType::Bang => Rule::init(Prefix::Unary, Infix::None, Precedence::None),
        TokenType::BangEqual => Rule::init(Prefix::None, Infix::Binary, Precedence::Equality),
        TokenType::Equal => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::EqualEqual => Rule::init(Prefix::None, Infix::Binary, Precedence::Equality),
        TokenType::Greater => Rule::init(Prefix::None, Infix::Binary, Precedence::Comparison),
        TokenType::GreaterEqual => Rule::init(Prefix::None, Infix::Binary, Precedence::Comparison),
        TokenType::Less => Rule::init(Prefix::None, Infix::Binary, Precedence::Comparison),
        TokenType::LessEqual => Rule::init(Prefix::None, Infix::Binary, Precedence::Comparison),
        TokenType::Identifier => Rule::init(Prefix::Variable, Infix::None, Precedence::None),
        TokenType::String => Rule::init(Prefix::String, Infix::None, Precedence::None),
        TokenType::Number => Rule::init(Prefix::Number, Infix::None, Precedence::None),
        TokenType::And => Rule::init(Prefix::None, Infix::And, Precedence::And),
        TokenType::Class => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::Else => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::False => Rule::init(Prefix::Literal, Infix::None, Precedence::None),
        TokenType::For => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::Fun => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::If => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::Nil => Rule::init(Prefix::Literal, Infix::None, Precedence::None),
        TokenType::Or => Rule::init(Prefix::None, Infix::Or, Precedence::Or),
        TokenType::Print => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::Return => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::Super => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::This => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::True => Rule::init(Prefix::Literal, Infix::None, Precedence::None),
        TokenType::Var => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::While => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::Error => Rule::init(Prefix::None, Infix::None, Precedence::None),
        TokenType::Eof => Rule::init(Prefix::None, Infix::None, Precedence::None),
    }
}

impl<'a> Parser<'a> {
    pub fn init(source: &'a str) -> Self {
        Parser {
            scanner: Scanner::init(source),
            compiler: Compiler::new(),
            previous: Token {
                kind: TokenType::Error,
                lexeme: "before file",
                line: 0,
            },
            current: Token {
                kind: TokenType::Error,
                lexeme: "before file",
                line: 0,
            },
            chunk: None,
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn compile(&mut self) -> Option<Chunk> {
        self.chunk = Some(Chunk::new());

        self.advance();
        while !self.matches(TokenType::Eof) {
            self.declaration();
        }
        self.consume(TokenType::Eof, "Expect end of expression.");
        self.end_compiler();

        if self.had_error {
            None
        } else {
            Some(self.chunk.as_ref().unwrap().clone())
        }
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        self.chunk.as_ref().unwrap().disassemble("code");
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }

    fn matches(&mut self, kind: TokenType) -> bool {
        if self.current.kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) {
        self.previous = self.current;
        loop {
            self.current = self.scanner.scan_token();
            if self.current.kind != TokenType::Error {
                break;
            }

            self.error_at_current(self.current.lexeme);
        }
    }

    fn declaration(&mut self) {
        if self.matches(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.kind != TokenType::Eof {
            if self.previous.kind == TokenType::Semicolon {
                return;
            }
            match self.current.kind {
                TokenType::Class
                | TokenType::Fun
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Var
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }
            self.advance();
        }
    }

    fn var_declaration(&mut self) {
        self.parse_variable("Expect variable name.");
        if self.matches(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil);
        }

        let last = self.compiler.locals.last_mut().unwrap();
        last.depth = Some(self.compiler.scope_depth);

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );
    }

    fn parse_variable(&mut self, msg: &str) {
        self.consume(TokenType::Identifier, msg);

        self.declare_variable();
    }

    fn declare_variable(&mut self) {
        let t = self.previous.clone();
        if self.compiler.variable_already_declared(&t) {
            self.error_at_current("Already a variable with this name in this scope.");
        }
        self.compiler.add_local(t);
    }

    fn statement(&mut self) {
        if self.matches(TokenType::Print) {
            self.print_statement();
        } else if self.matches(TokenType::LeftBrace) {
            self.block();
        } else if self.matches(TokenType::If) {
            self.if_statement();
        } else if self.matches(TokenType::While) {
            self.while_statement();
        } else if self.matches(TokenType::For) {
            self.for_statement();
        } else {
            self.expression_statement();
        }
    }

    fn block(&mut self) {
        self.begin_scope();
        while self.current.kind != TokenType::RightBrace && self.current.kind != TokenType::Eof {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
        self.end_scope();
    }

    fn begin_scope(&mut self) {
        self.compiler.begin_scope();
    }

    fn end_scope(&mut self) {
        self.compiler.end_scope();
        let removed_from_stack = self.compiler.locals_removed_from_stack();
        for _ in 0..removed_from_stack {
            self.emit_byte(OpCode::Pop);
        }
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print);
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let then_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);

        self.statement();

        let else_jump = self.emit_jump(OpCode::Jump);
        self.patch_jump(then_jump);
        self.emit_byte(OpCode::Pop);

        if self.matches(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn while_statement(&mut self) {
        let loop_start = self.current_chunk().size();
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let end_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);

        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(end_jump);
        self.emit_byte(OpCode::Pop);
    }

    fn for_statement(&mut self) {
        self.begin_scope();

        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");
        if self.matches(TokenType::Semicolon) {
            // No initializer.
        } else if self.matches(TokenType::Var) {
        self.var_declaration();
        } else {
        self.expression_statement();
        }

        let mut loop_start = self.current_chunk().size();
        let mut exit_jump = None;
        if !self.matches(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");

            exit_jump = Some(self.emit_jump(OpCode::JumpIfFalse));
            self.emit_byte(OpCode::Pop)
        }

        if !self.matches(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump);
            let increment_start = self.current_chunk().size();
            self.expression();
            self.emit_byte(OpCode::Pop);
            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");
        
            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
          }

        self.statement();

        self.emit_loop(loop_start);


        if let Some(jump) = exit_jump {
            self.patch_jump(jump);
            self.emit_byte(OpCode::Pop);
        }

        self.end_scope();
    }

    fn emit_loop(&mut self, offset: u32) {
        self.emit_byte(OpCode::Loop);
        let line = self.current.line;
        let chunk = self.current_chunk();
        let jump = (chunk.size() as i64 + 4) - offset as i64;
        chunk.write_u32(jump as u32, line);

    }

    fn emit_jump(&mut self, code: OpCode) -> u32 {
        self.emit_byte(code);
        let line = self.current.line;
        let chunk = self.current_chunk();
        chunk.write_u32(u32::MAX, line);
        return chunk.size() - 4;
    }

    fn patch_jump(&mut self, offset: u32) {
        let chunk = self.current_chunk();
        let jump = chunk.size() - offset - 4;

        for (i, b) in jump.to_be_bytes().iter().enumerate() {
            chunk.code[offset as usize + i] = *b;
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn parse_precedence(&mut self, prec: Precedence) {
        self.advance();
        let can_assign = prec <= Precedence::Assignment;
        match get_rule(&self.previous.kind).prefix {
            Prefix::None => {
                self.error_at_current("Expect expression.");
                return;
            }
            Prefix::Variable => self.variable(can_assign),
            Prefix::Literal => self.literal(),
            Prefix::Grouping => self.grouping(),
            Prefix::Unary => self.unary(),
            Prefix::Number => self.number(),
            Prefix::String => self.string(),
        }

        while prec <= get_rule(&self.current.kind).precedence {
            self.advance();
            match get_rule(&self.previous.kind).infix {
                Infix::None => {}
                Infix::Binary => self.binary(),
                Infix::And => self.and(),
                Infix::Or => self.or(),
            }
        }

        if can_assign && self.matches(TokenType::Equal) {
            self.error_at_current("Invalid assigment target.");
        }
    }

    fn string(&mut self) {
        let s = self.previous.lexeme;
        self.emit_constant(Value::string(&s[1..s.len() - 1]));
    }

    fn variable(&mut self, can_assign: bool) {
        if let Some(i) = self.resolve_local(&self.previous.lexeme) {
            let line = self.previous.line;
            if can_assign && self.matches(TokenType::Equal) {
                self.expression();
                self.emit_byte(OpCode::SetLocal);
                let last = self.compiler.locals.last_mut().unwrap();
                last.depth = Some(self.compiler.scope_depth);
            } else {
                self.emit_byte(OpCode::GetLocal);
            }
            let chunk = self.current_chunk();
            chunk.write_u32(i, line);
        } else {
            self.error_at_current(&*format!("Unknown variable '{}'.", self.previous.lexeme));
        }
    }

    fn resolve_local(&mut self, name: &str) -> Option<u32> {
        for (i, local) in self.compiler.locals.iter().enumerate().rev() {
            if name == local.token.lexeme {
                if local.depth.is_none() {
                    self.error_at_current("Can't read local variable in its own initializer.");
                }
                return Some(i as u32);
            }
        }
        None
    }

    fn literal(&mut self) {
        match self.previous.kind {
            TokenType::Nil => self.emit_constant(Value::nil()),
            TokenType::False => self.emit_constant(Value::from_bool(false)),
            TokenType::True => self.emit_constant(Value::from_bool(true)),
            _ => panic!("Unsupported literal."),
        }
    }

    fn number(&mut self) {
        let v = f64::from_str(self.previous.lexeme).unwrap();
        self.emit_constant(Value::from_number(v));
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self) {
        let op_type = self.previous.kind;

        self.parse_precedence(Precedence::Unary);

        match op_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            TokenType::Bang => self.emit_byte(OpCode::Not),
            other => panic!("unknown unary operator: {:?}", other),
        }
    }

    fn and(&mut self) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_byte(OpCode::Pop);
        self.parse_precedence(Precedence::And);

        self.patch_jump(end_jump);
    }

    fn or(&mut self) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse);
        let end_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(else_jump);
        self.emit_byte(OpCode::Pop);

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn binary(&mut self) {
        let op_type = self.previous.kind;
        let rule = get_rule(&op_type);
        self.parse_precedence(rule.precedence.next());

        match op_type {
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Substract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            TokenType::BangEqual => {
                self.emit_byte(OpCode::Equal);
                self.emit_byte(OpCode::Not);
            }
            TokenType::EqualEqual => self.emit_byte(OpCode::Equal),
            TokenType::Less => self.emit_byte(OpCode::Less),
            TokenType::LessEqual => {
                self.emit_byte(OpCode::Greater);
                self.emit_byte(OpCode::Not);
            }
            TokenType::Greater => self.emit_byte(OpCode::Greater),
            TokenType::GreaterEqual => {
                self.emit_byte(OpCode::Less);
                self.emit_byte(OpCode::Not);
            }
            other => panic!("unknown binary operator: {:?}", other),
        }
    }

    fn emit_constant(&mut self, v: Value) {
        let line = self.previous.line;
        let chunk = self.current_chunk();
        let i = chunk.add_constant(v);
        chunk.write_chunk(OpCode::Constant, line);
        chunk.write_u32(i, line);
    }

    fn consume(&mut self, kind: TokenType, msg: &str) {
        if self.current.kind == kind {
            self.advance();
        } else {
            self.error_at_current(msg);
        }
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        self.chunk.as_mut().unwrap()
    }

    fn emit_byte(&mut self, b: OpCode) {
        let line = self.previous.line;
        let chunk = self.current_chunk();
        chunk.write_chunk(b, line);
    }

    fn error_at_current(&mut self, lexeme: &str) {
        let at = self.current;
        self.error_at(&at, lexeme);
    }

    fn error_at(&mut self, at: &Token<'_>, msg: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        eprint!("[line {}] Error", at.line);
        if at.kind == TokenType::Eof {
            eprint!(" at end");
        } else if at.kind == TokenType::Error {
        } else {
            eprint!(" at {}", at.lexeme);
        }

        eprintln!(": {}", msg);
        self.had_error = true;
    }
}

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

    fn scan_token(&mut self) -> Token<'a> {
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

    fn identifier(&mut self) -> Token<'a> {
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
                        _ => TokenType::Identifier,
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
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'v' => self.check_keyword(1, 2, "ar", TokenType::Var),
            'w' => self.check_keyword(1, 4, "hile", TokenType::While),
            _ => TokenType::Identifier,
        }
    }

    fn check_keyword(&self, start: usize, length: usize, rest: &str, kind: TokenType) -> TokenType {
        if self.current - self.start == start + length
            && rest == &self.source[self.start + start..self.start + start + length]
        {
            kind
        } else {
            TokenType::Identifier
        }
    }

    fn number(&mut self) -> Token<'a> {
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

    fn string(&mut self) -> Token<'a> {
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
            if self.is_at_end() {
                return;
            }
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

    fn make_token(&self, kind: TokenType) -> Token<'a> {
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

#[derive(Clone, Copy, Debug)]
struct Token<'a> {
    pub kind: TokenType,
    pub lexeme: &'a str,
    pub line: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    fn next(&self) -> Self {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => panic!("no precendence above primary!"),
        }
    }
}
