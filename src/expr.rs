use crate::token::Token;
use crate::token::Object;

pub enum Expr {
    Literal(Object),
    Grouping(Box<Expr>),
    Unary(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
}

// fn ast_printer(expr: &Expr) -> String {
//     match expr {
//         Expr::Literal(obj) => format!("{:?}", obj),
//         Expr::Grouping(expr) => format!("(group {})", ast_printer(expr)),
//         Expr::Unary(token, expr) => format!("({} {})", token.lexeme, ast_printer(expr)),
//         Expr::Binary(left, token, right) => format!("({} {} {})", token.lexeme, ast_printer(left), ast_printer(right)),
//     }
// }