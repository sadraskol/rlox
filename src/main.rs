use std::env::args;
use std::io::prelude::*;

mod token;

fn main() {
    for a in args() {
        println!("{}", a);
    }
    if args().count() > 2 {
        println!("Usage: rlox [script]");
        std::process::exit(64);
    } else if args().count() == 2 {
        let mut args = args();
        args.next();
        run_file(args.next().unwrap());
    } else {
        run_prompt();
    }
}

fn run_file(f_name: String) {
    let source = std::fs::read_to_string(f_name).unwrap();
    let mut prog = Lox::new();
    prog.run(source);
    if prog.had_error {
        std::process::exit(65);
    }
}

fn run_prompt() {
    let stdin = std::io::stdin();

    for line in stdin.lock().lines() {
        Lox::new().run(line.unwrap());
    }
}

struct Lox {
    had_error: bool
}

impl Lox {
    fn new() -> Self {
        Lox { had_error: false }
    }

    fn run(&mut self, source: String) {
        print!("{}", source);
    }

    fn error(&mut self, line: usize, message: String) {
        self.report(line, "".to_string(), message);
    }

    fn report(&mut self, line: usize, were: String, message: String) {
        eprintln!("[line {}] Error{}: {}", line, were, message);
        self.had_error = true;
    }
}
