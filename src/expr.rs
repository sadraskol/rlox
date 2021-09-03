use crate::token::Object;
use crate::token::Token;

#[derive(Debug)]
pub enum Stmt {
    Print(Box<Expr>,),
    Expr(Box<Expr>),
    Var(Box<Token>, Option<Box<Expr>>),
}

#[derive(Debug)]
pub enum Expr {
    Grouping(Box<Expr>),
    Unary(Box<Token>, Box<Expr>),
    Binary(Box<Expr>, Box<Token>, Box<Expr>),
    Literal(Object),
    Variable(Box<Token>),
}
