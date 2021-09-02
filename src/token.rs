#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
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
    Fun,
    For,
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

    Eof,
}

#[derive(Clone, Debug)]
pub enum Object {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenType,
    pub lexeme: String,
    pub literal: Option<Object>,
    pub line: usize,
}

impl Token {
    pub fn new(kind: TokenType, lexeme: String, literal: Option<Object>, line: usize) -> Self {
        Token {
            kind,
            lexeme,
            literal,
            line,
        }
    }
}
