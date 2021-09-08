use crate::token::Object;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expr(Box<Expr>),
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    While(Box<Expr>, Box<Stmt>),
    Print(Box<Expr>),
    Return(Box<Token>, Box<Expr>),
    Var(Box<Token>, Option<Box<Expr>>),
    Fn(Box<Token>, Vec<Box<Token>>, Vec<Stmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Assign(Box<Token>, Box<Expr>),
    Binary(Box<Expr>, Box<Token>, Box<Expr>),
    Call(Box<Expr>, Box<Token>, Vec<Box<Expr>>),
    Grouping(Box<Expr>),
    Logical(Box<Expr>, Box<Token>, Box<Expr>),
    Unary(Box<Token>, Box<Expr>),
    Literal(Object),
    Variable(Box<Token>),
}
