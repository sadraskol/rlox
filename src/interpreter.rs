use crate::expr::Expr;
use crate::expr::Stmt;
use crate::token::Object;
use crate::token::TokenType;
use crate::token::Token;
use crate::LoxError;

use std::collections::HashMap;

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, Object>,
}

impl Environment {
    fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    fn get(&self, token: &Token) -> Object {
        self.values.get(&token.lexeme)
            .expect(&*format!("Undefined variable '{}'.", token.lexeme)).clone()
    }
}

#[derive(Default)]
pub struct Interpreter {
    env: Environment,
}

type Result<T> = crate::Result<T>;

fn is_thruthy(o: &Object) -> bool {
    match o {
        Object::Nil => false,
        Object::Bool(b) => *b,
        _ => true,
    }
}

fn is_equal(left: &Object, right: &Object) -> bool {
    if left == &Object::Nil && right == &Object::Nil {
        true
    } else if left == &Object::Nil {
        false
    } else {
        left == right
    }
}

fn checked_number(o: Object) -> Option<f64> {
    if let Object::Number(n) = o {
        Some(n)
    } else {
        None
    }
}

fn checked_string(o: Object) -> Option<String> {
    if let Object::String(s) = o {
        Some(s)
    } else {
        None
    }
}

impl Interpreter {
    pub fn interpret_statement(&mut self, statement: Stmt) -> Result<()> {
        match statement {
            Stmt::Expr(expr) => { self.interpret(expr)?; }
            Stmt::Print(expr) => println!("{}", self.interpret(expr)?),
            Stmt::Var(token, expr) => { 
                let init = if let Some(e) = expr {
                    self.interpret(e)?
                } else {
                    Object::Nil
                };
                self.env.define(token.lexeme, init);
            }
        };
        Ok(())
    }

    pub fn interpret(&self, expr: Box<Expr>) -> Result<Object> {
        match *expr {
            Expr::Variable(name) => Ok(self.env.get(&name)),
            Expr::Literal(obj) => Ok(obj),
            Expr::Grouping(ex) => self.interpret(ex),
            Expr::Unary(op, right) => {
                let right = self.interpret(right)?;

                match op.kind {
                    TokenType::Bang => {
                        if is_thruthy(&right) {
                            Ok(Object::Bool(false))
                        } else {
                            Ok(Object::Bool(true))
                        }
                    }
                    TokenType::Minus => {
                        let n = checked_number(right)
                            .ok_or_else(|| LoxError::error_tok(op, "Operand must be a number.".to_string()))?;
                        Ok(Object::Number(-n))
                    }
                    _ => Err(LoxError::error_tok(op, "Unknown unary operator.".to_string())),
                }
            }
            Expr::Binary(left, op, right) => {
                let left = self.interpret(left)?;
                let right = self.interpret(right)?;

                match op.kind {
                    TokenType::Minus => {
                        let l = checked_number(left).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        let r = checked_number(right).ok_or_else(|| LoxError::error_tok(op, "Operands must be numbers.".to_string()))?;
                        Ok(Object::Number(l - r))
                    }
                    TokenType::Slash => {
                        let l = checked_number(left).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        let r = checked_number(right).ok_or_else(|| LoxError::error_tok(op, "Operands must be numbers.".to_string()))?;
                        Ok(Object::Number(l / r))
                    }
                    TokenType::Star => {
                        let l = checked_number(left).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        let r = checked_number(right).ok_or_else(|| LoxError::error_tok(op, "Operands must be numbers.".to_string()))?;
                        Ok(Object::Number(l * r))
                    }
                    TokenType::Plus => {
                        if let Object::Number(l) = left {
                            let r = checked_number(right).ok_or_else(|| LoxError::error_tok(op, "Operands must two numbers or two strings.".to_string()))?;
                            Ok(Object::Number(l + r))
                        } else if let Object::String(l) = left {
                            let r = checked_string(right).ok_or_else(|| LoxError::error_tok(op, "Operands must two numbers or two strings.".to_string()))?;
                            Ok(Object::String(format!("{}{}", l, r)))
                        } else {
                            Err(LoxError::error_tok(op, "Operands must two numbers or two strings.".to_string()))
                        }
                    }
                    TokenType::Greater => {
                        let l = checked_number(left).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        let r = checked_number(right).ok_or_else(|| LoxError::error_tok(op, "Operands must be numbers.".to_string()))?;
                        Ok(Object::Bool(l > r))
                    }
                    TokenType::GreaterEqual => {
                        let l = checked_number(left).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        let r = checked_number(right).ok_or_else(|| LoxError::error_tok(op, "Operands must be numbers.".to_string()))?;
                        Ok(Object::Bool(l >= r))
                    }
                    TokenType::Less => {
                        let l = checked_number(left).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        let r = checked_number(right).ok_or_else(|| LoxError::error_tok(op, "Operands must be numbers.".to_string()))?;
                        Ok(Object::Bool(l < r))
                    }
                    TokenType::LessEqual => {
                        let l = checked_number(left).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        let r = checked_number(right).ok_or_else(|| LoxError::error_tok(op, "Operands must be numbers.".to_string()))?;
                        Ok(Object::Bool(l <= r))
                    }
                    TokenType::BangEqual => Ok(Object::Bool(!is_equal(&left, &right))),
                    TokenType::EqualEqual => Ok(Object::Bool(is_equal(&left, &right))),
                    _ => Ok(Object::Nil),
                }
            }
        }
    }
}
