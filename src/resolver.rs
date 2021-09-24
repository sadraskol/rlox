use crate::interpreter::InterpreterError;
use crate::expr::Stmt;
use crate::expr::Expr;
use crate::token::Token;
use crate::LoxError;

use std::collections::HashMap;

type Result<T> = crate::Result<T>;

#[derive(Clone, Copy, PartialEq)]
pub enum FunctionType { None, Function }

pub struct Resolver {
    stack: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            stack: vec![],
            current_function: FunctionType::None,
        }
    }

    pub fn interpret_statement(
        &mut self,
        statement: &mut Stmt,
    ) -> std::result::Result<(), InterpreterError> {
        match statement {
            Stmt::Expr(expr) => {
                self.interpret(expr)?;
            }
            Stmt::If(expr, then, otherwise) => {
                self.interpret(expr)?;
                self.interpret_statement(then)?;
                if let Some(otherwise) = otherwise {
                    self.interpret_statement(otherwise)?;
                }
            }
            Stmt::Print(expr) => {
                self.interpret(expr)?;
            }
            Stmt::Return(tok, expr) => {
                if self.current_function == FunctionType::None {
                    return Err(InterpreterError::Lox(LoxError::error_tok(tok, "Can't return from top-level code.".to_string())));
                }
                self.interpret(expr)?;
            }
            Stmt::While(cond, body) => {
                self.interpret(cond)?;
                self.interpret_statement(body)?;
            }
            Stmt::Block(stmts) => {
                self.begin_scope();
                self.resolve_all(stmts)?;
                self.end_scope();
            }
            Stmt::Var(tok, init) => {
                self.declare(tok)?;
                if let Some(init) = init {
                    self.interpret(init)?;
                }
                self.define(tok);
            }
            Stmt::Fn(name, args, body) => {
                self.declare(name)?;
                self.define(name);
                self.resolve_function(args, body)?;
            }
        }
        Ok(())
    }

    fn resolve_function(&mut self, args: &Vec<Token>, body: &mut Vec<Stmt>) -> std::result::Result<(), InterpreterError> {
        let enclosing_type = self.current_function;
        self.current_function = FunctionType::Function;

        self.begin_scope();
        for param in args {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_all(body)?;
        self.end_scope();

        self.current_function = enclosing_type;
        Ok(())
    }

    fn declare(&mut self, name: &Token) -> Result<()> {
        if !self.stack.is_empty() {
            let last = self.stack.last_mut().unwrap();
            if last.get(&name.lexeme).is_some() {
                return Err(LoxError::error_tok(name, "Already a variable with this name in current scope".to_string()));
            }
            last.insert(name.lexeme.clone(), false);
        }
        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if !self.stack.is_empty() {
            let last = self.stack.last_mut().unwrap();
            last.insert(name.lexeme.clone(), true);
        }
    }

    pub fn resolve_all(&mut self, statements: &mut Vec<Stmt>) -> std::result::Result<(), InterpreterError> {
        for statement in statements {
            self.interpret_statement(statement)?;
        }
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.stack.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.stack.pop();
    }

    fn interpret(&mut self, expr: &mut Expr) -> Result<()> {
        match expr {
            Expr::Variable(tok, depth) => {
                if !self.stack.is_empty() && !self.stack.last().unwrap().get(&tok.lexeme).unwrap_or(&true) {
                    Err(LoxError::error_tok(tok, "Can't read local variable in its own initializer.".to_string()))
                } else {
                    self.resolve_local(tok, depth);
                    Ok(())
                }
            }
            Expr::Assign(name, val, depth) => {
                self.interpret(val)?;
                self.resolve_local(name, depth);
                Ok(())
            }
            Expr::Binary(left, _op, right) => {
                self.interpret(left)?;
                self.interpret(right)?;
                Ok(())
            }
            Expr::Call(callee, _tok, args) => {
                self.interpret(callee)?;
                for arg in args {
                    self.interpret(arg)?;
                }
                Ok(())
            }
            Expr::Grouping(expr) => {
                self.interpret(expr)?;
                Ok(())
            }
            Expr::Literal(_) => {
                Ok(())
            }
            Expr::Logical(left, _op, right) => {
                self.interpret(left)?;
                self.interpret(right)?;
                Ok(())
            }
            Expr::Unary(_op, right) => {
                self.interpret(right)?;
                Ok(())
            }
        }
    }

    fn resolve_local(&mut self, name: &Token, depth: &mut Option<usize>) {
        for (i, scope) in self.stack.iter().enumerate().rev() {
            if scope.get(&name.lexeme).is_some() {
                *depth = Some(i);
                return;
            }
        }
    }
}