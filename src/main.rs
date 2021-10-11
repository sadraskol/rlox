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
    let mut chunks = Chunk::new();

    let op_1 = chunks.add_constant(1.2);
    chunks.write_chunk(OpCode::OpConstant, 123);
    chunks.write_index(op_1, 123);

    let op_2 = chunks.add_constant(3.4);
    chunks.write_chunk(OpCode::OpConstant, 123);
    chunks.write_index(op_2, 123);

    chunks.write_chunk(OpCode::OpAdd, 123);

    let op_3 = chunks.add_constant(5.6);
    chunks.write_chunk(OpCode::OpConstant, 123);
    chunks.write_index(op_3, 123);

    chunks.write_chunk(OpCode::OpDivide, 123);
    chunks.write_chunk(OpCode::OpNegate, 123);

    chunks.write_chunk(OpCode::OpReturn, 123);

    chunks.disassemble("test chunk");

    let mut vm = VM::new(chunks);
    vm.run();
}