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
                    println!("'{}'", self.pop());
                    return InterpretResult::Ok;
                }
                OpCode::OpConstant => {
                    let bytes = &self.chunk.code[self.ip..self.ip + 4];
                    self.ip += 4;
                    let sized_bytes = bytes.try_into().unwrap();
                    let index = u32::from_be_bytes(sized_bytes);
                    let constant = self.chunk.constants[index as usize];
                    self.stack.push(constant);
                }
                OpCode::OpDivide => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a / b);
                }
                OpCode::OpAdd => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a + b);
                }
                OpCode::OpNegate => {
                    let neg = -self.pop();
                    self.push(neg);
                }
                OpCode::OpMultiply => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a * b);
                }
                OpCode::OpSubstract => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a - b);
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
    std::fs::read_file_string(std::env::args
}
