use std::env::args;
use std::convert::TryInto;
use crate::compiler::Parser;
use crate::chunk::Chunk;
use crate::chunk::Value;
use crate::chunk::OpCode;


mod compiler;
mod chunk;

struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

enum InterpretResult {
    Ok,
    CompileError
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
                    println!("'{:?}'", self.pop());
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
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::from_number(a.as_number() / b.as_number()));
                }
                OpCode::OpAdd => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::from_number(a.as_number() + b.as_number()));
                }
                OpCode::OpNegate => {
                    let neg = self.pop();
                    self.push(Value::from_number(-neg.as_number()));
                }
                OpCode::OpMultiply => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::from_number(a.as_number() * b.as_number()));
                }
                OpCode::OpSubstract => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::from_number(a.as_number() - b.as_number()));
                }
            }
        }
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let mut compiler = Parser::init(source);
        let chunk = compiler.compile();

        if let Some(chunk) = chunk {
            self.chunk = chunk;
            self.ip = 0;
            self.run()
        } else {
            InterpretResult::CompileError
        }
    }
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
