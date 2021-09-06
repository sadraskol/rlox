use crate::token::Object;
use crate::token::Token;

#[derive(Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expr(Box<Expr>),
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    While(Box<Expr>, Box<Stmt>),
    Print(Box<Expr>,),
    Var(Box<Token>, Option<Box<Expr>>),
}

#[derive(Debug)]
pub enum Expr {
    Assign(Box<Token>, Box<Expr>),
    Binary(Box<Expr>, Box<Token>, Box<Expr>),
    Logical(Box<Expr>, Box<Token>, Box<Expr>),
    Grouping(Box<Expr>),
    Unary(Box<Token>, Box<Expr>),
    Literal(Object),
    Variable(Box<Token>),
}
