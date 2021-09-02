use std::env::args;

use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::token::Token;
use crate::token::TokenType;

mod expr;
mod parser;
mod scanner;
mod token;

fn main() {
    for a in args() {
        println!("{}", a);
    }

    match args().count() {
        2 => {
            let mut args = args();
            args.next();
            run_file(args.next().unwrap());
        }
        _ => {
            println!("Usage: rlox [script]");
            std::process::exit(64);
        }
    }
}

fn run_file(f_name: String) {
    let source = std::fs::read_to_string(f_name).unwrap();
    let prog = Lox::new();
    if prog.run(source).is_err() {
        std::process::exit(65);
    }
}

struct Lox;

#[derive(Debug)]
pub struct LoxError {
    line: usize,
    location: String,
    message: String,
}

impl LoxError {
    pub fn error<Any>(line: usize, message: String) -> Result<Any> {
        Self::report(line, "".to_string(), message)
    }

    pub fn error_tok<Any>(token: Box<Token>, message: String) -> Result<Any> {
        if token.kind == TokenType::Eof {
            Self::report(token.line, " at end".to_string(), message)
        } else {
            Self::report(token.line, format!(" at '{}'", token.lexeme), message)
        }
    }

    fn report<Any>(line: usize, location: String, message: String) -> Result<Any> {
        eprintln!("[line {}] Error{}: {}", line, location, message);
        Err(vec![LoxError {
            line,
            location,
            message,
        }])
    }
}

pub type Result<T> = std::result::Result<T, Vec<LoxError>>;

impl Lox {
    fn new() -> Self {
        Lox
    }

    fn run(&self, source: String) -> Result<()> {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let expr = parser.parse()?;

        println!("{:?}", expr);
        Ok(())
    }
}
