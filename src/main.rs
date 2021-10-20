use crate::chunk::Chunk;
use crate::chunk::OpCode;
use crate::chunk::Value;
use crate::compiler::Parser;
use std::convert::TryInto;
use std::env::args;

mod chunk;
mod compiler;

struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

impl VM {
    fn new(chunk: Chunk) -> Self {
        VM {
            chunk,
            ip: 0,
            stack: vec![],
        }
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn push(&mut self, v: Value) {
        self.stack.push(v);
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            let instruction = self.chunk.code[self.ip];
            self.ip += 1;
            match instruction.into() {
                OpCode::OpReturn => {
                    println!("'{}'", self.pop());
                    return InterpretResult::Ok;
                }
                OpCode::OpConstant => {
                    let bytes = &self.chunk.code[self.ip..self.ip + 4];
                    self.ip += 4;
                    let sized_bytes = bytes.try_into().unwrap();
                    let index = u32::from_be_bytes(sized_bytes);
                    let constant = &self.chunk.constants[index as usize];
                    self.stack.push(constant.clone());
                }
                OpCode::OpDivide => {
                    if !self.peek(0).is_number() || !self.peek(0).is_number() {
                        self.runtime_error("Operands must be numbers.");
                        return InterpretResult::RuntimeError;
                    }
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::from_number(a.as_number() / b.as_number()));
                }
                OpCode::OpAdd => {
                    if self.peek(0).is_string() && self.peek(1).is_string() {
                        self.concatenate();
                    } else if self.peek(0).is_number() && self.peek(0).is_number() {
                        let b = self.pop();
                        let a = self.pop();
                        self.push(Value::from_number(a.as_number() + b.as_number()));
                    } else {
                        self.runtime_error("Operands must be two numbers or two strings.");
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::OpNegate => {
                    if !self.peek(0).is_number() {
                        self.runtime_error("Operand must be a number.");
                        return InterpretResult::RuntimeError;
                    }
                    let neg = self.pop();
                    self.push(Value::from_number(-neg.as_number()));
                }
                OpCode::OpMultiply => {
                    if !self.peek(0).is_number() || !self.peek(0).is_number() {
                        self.runtime_error("Operands must be numbers.");
                        return InterpretResult::RuntimeError;
                    }
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::from_number(a.as_number() * b.as_number()));
                }
                OpCode::OpSubstract => {
                    if !self.peek(0).is_number() || !self.peek(0).is_number() {
                        self.runtime_error("Operands must be numbers.");
                        return InterpretResult::RuntimeError;
                    }
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::from_number(a.as_number() - b.as_number()));
                }
                OpCode::OpNot => {
                    if !self.peek(0).is_bool() {
                        self.runtime_error("Operand must be a bool.");
                        return InterpretResult::RuntimeError;
                    }
                    let b = self.pop();
                    self.push(Value::from_bool(!b.as_bool()));
                }
                OpCode::OpEqual => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::from_bool(a == b));
                }
                OpCode::OpLess => {
                    if !self.peek(0).is_number() || !self.peek(0).is_number() {
                        self.runtime_error("Operands must be numbers.");
                        return InterpretResult::RuntimeError;
                    }
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::from_bool(a.as_number() < b.as_number()));
                }
                OpCode::OpGreater => {
                    if !self.peek(0).is_number() || !self.peek(0).is_number() {
                        self.runtime_error("Operands must be numbers.");
                        return InterpretResult::RuntimeError;
                    }
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::from_bool(a.as_number() > b.as_number()));
                }
            }
        }
    }

    fn concatenate(&mut self) {
        let b = self.pop();
        let a = self.pop();

        let mut buf: String = a.as_str().to_string();
        buf.push_str(&b.as_str());
        self.push(Value::string(&buf));
    }

    fn peek(&self, depth: usize) -> &Value {
        &self.stack[self.stack.len() - 1 - depth]
    }

    fn runtime_error(&mut self, msg: &str) {
        eprintln!("{}", msg);
        let instruction = self.ip - 1; // todo this size depends on the last instruction size
        eprintln!("[line {}] in script", self.chunk.lines[instruction]);
        self.reset_stack();
    }

    fn reset_stack(&mut self) {}
}

fn main() {
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
    let mut compiler = Parser::init(&source);
    let chunk = compiler.compile();

    if let Some(chunk) = chunk {
        let mut vm = VM {
            chunk,
            ip: 0,
            stack: vec![],
        };
        vm.run();
    } else {
    }
}
