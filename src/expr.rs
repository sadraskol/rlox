use crate::token::Object;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expr(Expr),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Print(Expr),
    Return(Token, Expr),
    Var(Token, Option<Expr>),
    Fn(Token, Vec<Token>, Vec<Stmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Assign(Token, Box<Expr>, Option<usize> /* depth of the variable */),
    Binary(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
    Grouping(Box<Expr>),
    Logical(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Literal(Object),
    Variable(Token, Option<usize> /* depth of the variable */),
}
