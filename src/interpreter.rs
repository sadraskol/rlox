use crate::expr::Expr;
use crate::expr::Stmt;
use crate::token::LoxFn;
use crate::token::Object;
use crate::token::Token;
use crate::token::TokenType;
use crate::LoxError;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, Default, PartialEq)]
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

    fn define(&self, name: String, value: Object) {
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

pub struct Interpreter {
    env: Environment
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::default();
        globals.define("clock".to_string(), Object::Callable(0, LoxFn::Clock));
        Interpreter {
            env: globals,
        }
    }
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

pub enum InterpreterError {
    Lox(LoxError),
    Return(Token, Object),
}

impl From<LoxError> for InterpreterError {
    fn from(err: LoxError) -> Self {
        InterpreterError::Lox(err)
    }
}

impl Interpreter {
    pub fn interpret_statement(
        &mut self,
        statement: &Stmt,
    ) -> std::result::Result<(), InterpreterError> {
        match statement {
            Stmt::Block(decls) => {
                self.execute_block(decls, Environment::new(self.env.clone()))?;
            }
            Stmt::Return(tok, expr) => {
                let value = self.interpret(expr)?;
                return Err(InterpreterError::Return(tok.clone(), value));
            }
            Stmt::Fn(name, args, body) => {
                let fun = Object::Callable(
                    args.len(),
                    LoxFn::UserDef(Box::new(name.clone()), args.clone(), body.clone(), self.env.clone()),
                );
                self.env.define(name.lexeme.clone(), fun);
            }
            Stmt::If(expr, then_branch, else_branch) => {
                if is_thruthy(&self.interpret(expr)?) {
                    self.interpret_statement(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.interpret_statement(else_branch)?;
                }
            }
            Stmt::While(expr, body) => {
                while is_thruthy(&self.interpret(expr)?) {
                    self.interpret_statement(body)?;
                }
            }
            Stmt::Expr(expr) => {
                self.interpret(expr)?;
            }
            Stmt::Print(expr) => println!("{}", self.interpret(expr)?),
            Stmt::Var(token, expr) => {
                let init = if let Some(e) = expr {
                    self.interpret(e)?
                } else {
                    Object::Nil
                };
                self.env.define(token.lexeme.clone(), init);
            }
        };
        Ok(())
    }

    fn execute_block(
        &mut self,
        decls: &[Stmt],
        env: Environment,
    ) -> std::result::Result<(), InterpreterError> {
        let previous = self.env.clone();

        self.env = env;

        for decl in decls {
            let res = self.interpret_statement(decl);
            if let Err(InterpreterError::Return(t, v)) = res {
                self.env = previous;
                return Err(InterpreterError::Return(t, v));
            } else {
                res?;
            }
        }

        self.env = previous;
        Ok(())
    }

    fn interpret(&mut self, expr: &Expr) -> Result<Object> {
        match expr {
            Expr::Assign(name, right) => {
                let value = self.interpret(right)?;
                self.env.assign(name, value.clone());
                Ok(value)
            }
            Expr::Variable(name) => Ok(self.env.get(name)),
            Expr::Literal(obj) => Ok(obj.clone()),
            Expr::Grouping(ex) => self.interpret(ex),
            Expr::Call(callee_expr, token, args) => {
                let callee = self.interpret(callee_expr)?;

                let mut arguments = vec![];
                for arg in args {
                    arguments.push(self.interpret(arg)?);
                }

                if let Object::Callable(arity, f) = callee {
                    if arity != arguments.len() {
                        Err(LoxError::error_tok(
                            token,
                            format!("Expected {} arguments but got {}.", arity, arguments.len()),
                        ))
                    } else {
                        self.call(f, arguments)
                    }
                } else {
                    Err(LoxError::error_tok(
                        token,
                        "Can only call functions and classes.".to_string(),
                    ))
                }
            }
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
                        let n = checked_number(right).ok_or_else(|| {
                            LoxError::error_tok(op, "Operand must be a number.".to_string())
                        })?;
                        Ok(Object::Number(-n))
                    }
                    _ => Err(LoxError::error_tok(
                        op,
                        "Unknown unary operator.".to_string(),
                    )),
                }
            }
            Expr::Logical(left, op, right) => {
                let left = self.interpret(left)?;

                if op.kind == TokenType::Or {
                    if is_thruthy(&left) {
                        return Ok(left);
                    }
                } else if !is_thruthy(&left) {
                    return Ok(left);
                }

                self.interpret(right)
            }
            Expr::Binary(left, op, right) => {
                let left = self.interpret(left)?;
                let right = self.interpret(right)?;

                match op.kind {
                    TokenType::Minus => {
                        let l = checked_number(left).ok_or_else(|| {
                            LoxError::error_tok(op, "Operands must be numbers.".to_string())
                        })?;
                        let r = checked_number(right).ok_or_else(|| {
                            LoxError::error_tok(op, "Operands must be numbers.".to_string())
                        })?;
                        Ok(Object::Number(l - r))
                    }
                    TokenType::Slash => {
                        let l = checked_number(left).ok_or_else(|| {
                            LoxError::error_tok(op, "Operands must be numbers.".to_string())
                        })?;
                        let r = checked_number(right).ok_or_else(|| {
                            LoxError::error_tok(op, "Operands must be numbers.".to_string())
                        })?;
                        Ok(Object::Number(l / r))
                    }
                    TokenType::Star => {
                        let l = checked_number(left).ok_or_else(|| {
                            LoxError::error_tok(op, "Operands must be numbers.".to_string())
                        })?;
                        let r = checked_number(right).ok_or_else(|| {
                            LoxError::error_tok(op, "Operands must be numbers.".to_string())
                        })?;
                        Ok(Object::Number(l * r))
                    }
                    TokenType::Plus => {
                        if let Object::Number(l) = left {
                            let r = checked_number(right).ok_or_else(|| {
                                LoxError::error_tok(
                                    op,
                                    "Operands must two numbers or two strings.".to_string(),
                                )
                            })?;
                            Ok(Object::Number(l + r))
                        } else if let Object::String(l) = left {
                            let r = checked_string(right).ok_or_else(|| {
                                LoxError::error_tok(
                                    op,
                                    "Operands must two numbers or two strings.".to_string(),
                                )
                            })?;
                            Ok(Object::String(format!("{}{}", l, r)))
                        } else {
                            Err(LoxError::error_tok(
                                op,
                                "Operands must two numbers or two strings.".to_string(),
                            ))
                        }
                    }
                    TokenType::Greater => {
                        let l = checked_number(left).ok_or_else(|| {
                            LoxError::error_tok(op, "Operands must be numbers.".to_string())
                        })?;
                        let r = checked_number(right).ok_or_else(|| {
                            LoxError::error_tok(op, "Operands must be numbers.".to_string())
                        })?;
                        Ok(Object::Bool(l > r))
                    }
                    TokenType::GreaterEqual => {
                        let l = checked_number(left).ok_or_else(|| {
                            LoxError::error_tok(op, "Operands must be numbers.".to_string())
                        })?;
                        let r = checked_number(right).ok_or_else(|| {
                            LoxError::error_tok(op, "Operands must be numbers.".to_string())
                        })?;
                        Ok(Object::Bool(l >= r))
                    }
                    TokenType::Less => {
                        let l = checked_number(left).ok_or_else(|| {
                            LoxError::error_tok(op, "Operands must be numbers.".to_string())
                        })?;
                        let r = checked_number(right).ok_or_else(|| {
                            LoxError::error_tok(op, "Operands must be numbers.".to_string())
                        })?;
                        Ok(Object::Bool(l < r))
                    }
                    TokenType::LessEqual => {
                        let l = checked_number(left).ok_or_else(|| {
                            LoxError::error_tok(op, "Operands must be numbers.".to_string())
                        })?;
                        let r = checked_number(right).ok_or_else(|| {
                            LoxError::error_tok(op, "Operands must be numbers.".to_string())
                        })?;
                        Ok(Object::Bool(l <= r))
                    }
                    TokenType::BangEqual => Ok(Object::Bool(!is_equal(&left, &right))),
                    TokenType::EqualEqual => Ok(Object::Bool(is_equal(&left, &right))),
                    _ => Ok(Object::Nil),
                }
            }
        }
    }

    fn call(&mut self, callee: LoxFn, arguments: Vec<Object>) -> Result<Object> {
        match callee {
            LoxFn::Clock => {
                let x = std::time::SystemTime::now()
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .unwrap();
                Ok(Object::Number(x.as_secs() as f64))
            }
            LoxFn::UserDef(_, args, body, closure) => {
                let env = Environment::new(closure);
                for i in 0..args.len() {
                    env.define(args[i].lexeme.clone(), arguments[i].clone());
                }

                match self.execute_block(&body, env) {
                    Err(InterpreterError::Return(_, v)) => Ok(v),
                    Err(InterpreterError::Lox(e)) => Err(e),
                    _ => Ok(Object::Nil),
                }
            }
        }
    }
}
