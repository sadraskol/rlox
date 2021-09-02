use crate::token::Object;
use crate::token::Token;

#[derive(Debug)]
pub enum Expr {
    Literal(Object),
    Grouping(Box<Expr>),
    Unary(Box<Token>, Box<Expr>),
    Binary(Box<Expr>, Box<Token>, Box<Expr>),
}
