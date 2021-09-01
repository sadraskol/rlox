use std::env::args;

use crate::scanner::Scanner;
use std::cell::Cell;

mod scanner;
mod token;
mod expr;

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
    prog.run(source);
    if prog.had_error.get() {
        std::process::exit(65);
    }
}

#[derive(Clone)]
pub struct Lox {
    had_error: Box<Cell<bool>>,
}

impl Lox {
    fn new() -> Self {
        Lox {
            had_error: Box::new(Cell::new(false)),
        }
    }

    fn run(&self, source: String) {
        let mut scanner = Scanner::new(source, self.clone());
        let tokens = scanner.scan_tokens();
        for token in tokens {
            println!("{:?}", token);
        }
    }

    pub fn error(&mut self, line: usize, message: String) {
        self.report(line, "".to_string(), message);
    }

    fn report(&mut self, line: usize, were: String, message: String) {
        eprintln!("[line {}] Error{}: {}", line, were, message);
        *self.had_error.get_mut() = true;
    }
}
