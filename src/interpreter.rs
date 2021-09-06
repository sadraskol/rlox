use crate::expr::Expr;
use crate::expr::Stmt;
use crate::token::Object;
use crate::token::TokenType;
use crate::token::Token;
use crate::LoxError;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone, Default)]
pub struct Environment {
    enclosing: Rc<RefCell<Option<Environment>>>,
    values: Rc<RefCell<HashMap<String, Object>>>,
}

impl Environment {
    fn new(enclosing: Environment) -> Self {
        Environment {
            enclosing: Rc::new(RefCell::new(Some(enclosing))),
            values: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    fn define(&mut self, name: String, value: Object) {
        self.values.borrow_mut().insert(name, value);
    }

    fn assign(&self, token: &Token, value: Object) {
        if self.values.borrow().contains_key(&token.lexeme) {
            self.values.borrow_mut().insert(token.lexeme.clone(), value);
        } else if let Some(enclosing) = &*self.enclosing.borrow() {
            enclosing.assign(token, value)
        } else {
            panic!("Undefined variable '{}'.", token.lexeme);
        }
    }

    fn get(&self, token: &Token) -> Object {
        let values = self.values.borrow();
        let res = values.get(&token.lexeme);
        if let Some(r) = res {
            r.clone()
        } else if let Some(enclosing) = &*self.enclosing.borrow() {
            enclosing.get(token)
        } else {
            panic!("Undefined variable '{}'.", token.lexeme);
        }
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
    pub fn interpret_statement(&mut self, statement: &Stmt) -> Result<()> {
        match statement {
            Stmt::Block(decls) => {
                self.execute_block(decls)?;
            }
            Stmt::If(expr, then_branch, else_branch) => {
                if is_thruthy(&self.interpret(&expr)?) {
                    self.interpret_statement(&then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.interpret_statement(&else_branch)?;
                }
            }
            Stmt::While(expr, body) => {
                while is_thruthy(&self.interpret(&expr)?) {
                    self.interpret_statement(&body)?;
                }
            }
            Stmt::Expr(expr) => { self.interpret(&expr)?; }
            Stmt::Print(expr) => println!("{}", self.interpret(&expr)?),
            Stmt::Var(token, expr) => { 
                let init = if let Some(e) = expr {
                    self.interpret(&e)?
                } else {
                    Object::Nil
                };
                self.env.define(token.lexeme.clone(), init);
            }
        };
        Ok(())
    }

    fn execute_block(&mut self, decls: &Vec<Stmt>) -> Result<()>{
        let previous = self.env.clone();

        self.env = Environment::new(self.env.clone());
        for decl in decls {
            self.interpret_statement(decl)?;
        }
        self.env = previous;
        Ok(())
    }

    fn interpret(&mut self, expr: &Expr) -> Result<Object> {
        match expr {
            Expr::Assign(name, right) => {
                let value = self.interpret(&right)?;
                self.env.assign(&name, value.clone());
                Ok(value)
            },
            Expr::Variable(name) => Ok(self.env.get(&name)),
            Expr::Literal(obj) => Ok(obj.clone()),
            Expr::Grouping(ex) => self.interpret(&ex),
            Expr::Unary(op, right) => {
                let right = self.interpret(&right)?;

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
                            .ok_or_else(|| LoxError::error_tok(op.clone(), "Operand must be a number.".to_string()))?;
                        Ok(Object::Number(-n))
                    }
                    _ => Err(LoxError::error_tok(op.clone(), "Unknown unary operator.".to_string())),
                }
            }
            Expr::Logical(left, op, right) => {
                let left = self.interpret(&left)?;

                if op.kind == TokenType::Or {
                    if is_thruthy(&left) {
                        return Ok(left);
                    }
                } else {
                    if !is_thruthy(&left) {
                        return Ok(left);
                    }
                }

                self.interpret(&right)
            }
            Expr::Binary(left, op, right) => {
                let left = self.interpret(&left)?;
                let right = self.interpret(&right)?;

                match op.kind {
                    TokenType::Minus => {
                        let l = checked_number(left).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        let r = checked_number(right).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        Ok(Object::Number(l - r))
                    }
                    TokenType::Slash => {
                        let l = checked_number(left).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        let r = checked_number(right).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        Ok(Object::Number(l / r))
                    }
                    TokenType::Star => {
                        let l = checked_number(left).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        let r = checked_number(right).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        Ok(Object::Number(l * r))
                    }
                    TokenType::Plus => {
                        if let Object::Number(l) = left {
                            let r = checked_number(right).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must two numbers or two strings.".to_string()))?;
                            Ok(Object::Number(l + r))
                        } else if let Object::String(l) = left {
                            let r = checked_string(right).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must two numbers or two strings.".to_string()))?;
                            Ok(Object::String(format!("{}{}", l, r)))
                        } else {
                            Err(LoxError::error_tok(op.clone(), "Operands must two numbers or two strings.".to_string()))
                        }
                    }
                    TokenType::Greater => {
                        let l = checked_number(left).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        let r = checked_number(right).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        Ok(Object::Bool(l > r))
                    }
                    TokenType::GreaterEqual => {
                        let l = checked_number(left).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        let r = checked_number(right).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        Ok(Object::Bool(l >= r))
                    }
                    TokenType::Less => {
                        let l = checked_number(left).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        let r = checked_number(right).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        Ok(Object::Bool(l < r))
                    }
                    TokenType::LessEqual => {
                        let l = checked_number(left).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
                        let r = checked_number(right).ok_or_else(|| LoxError::error_tok(op.clone(), "Operands must be numbers.".to_string()))?;
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
