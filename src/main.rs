use crate::chunk::Chunk;
use crate::chunk::OpCode;
use crate::chunk::Value;
use crate::compiler::Parser;
use std::convert::TryInto;
use std::env::args;
use std::collections::HashMap;

mod chunk;
mod compiler;

struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
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
            globals: HashMap::new(),
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
                OpCode::Return => {
                    return InterpretResult::Ok;
                }
                OpCode::Constant => {
                    let bytes = &self.chunk.code[self.ip..self.ip + 4];
                    self.ip += 4;
                    let sized_bytes = bytes.try_into().unwrap();
                    let index = u32::from_be_bytes(sized_bytes);
                    let constant = &self.chunk.constants[index as usize];
                    self.stack.push(constant.clone());
                }
                OpCode::Divide => {
                    if !self.peek(0).is_number() || !self.peek(0).is_number() {
                        self.runtime_error("Operands must be numbers.");
                        return InterpretResult::RuntimeError;
                    }
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::from_number(a.as_number() / b.as_number()));
                }
                OpCode::Add => {
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
                OpCode::Negate => {
                    if !self.peek(0).is_number() {
                        self.runtime_error("Operand must be a number.");
                        return InterpretResult::RuntimeError;
                    }
                    let neg = self.pop();
                    self.push(Value::from_number(-neg.as_number()));
                }
                OpCode::Multiply => {
                    if !self.peek(0).is_number() || !self.peek(0).is_number() {
                        self.runtime_error("Operands must be numbers.");
                        return InterpretResult::RuntimeError;
                    }
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::from_number(a.as_number() * b.as_number()));
                }
                OpCode::Substract => {
                    if !self.peek(0).is_number() || !self.peek(0).is_number() {
                        self.runtime_error("Operands must be numbers.");
                        return InterpretResult::RuntimeError;
                    }
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::from_number(a.as_number() - b.as_number()));
                }
                OpCode::Not => {
                    if !self.peek(0).is_bool() {
                        self.runtime_error("Operand must be a bool.");
                        return InterpretResult::RuntimeError;
                    }
                    let b = self.pop();
                    self.push(Value::from_bool(!b.as_bool()));
                }
                OpCode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::from_bool(a == b));
                }
                OpCode::Less => {
                    if !self.peek(0).is_number() || !self.peek(0).is_number() {
                        self.runtime_error("Operands must be numbers.");
                        return InterpretResult::RuntimeError;
                    }
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::from_bool(a.as_number() < b.as_number()));
                }
                OpCode::Greater => {
                    if !self.peek(0).is_number() || !self.peek(0).is_number() {
                        self.runtime_error("Operands must be numbers.");
                        return InterpretResult::RuntimeError;
                    }
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::from_bool(a.as_number() > b.as_number()));
                }
                OpCode::Print => {
                    println!("{}", self.pop().as_str());
                }
                OpCode::Nil => {
                    self.push(Value::Nil);
                }
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::DefineGlobal => {
                    let bytes = &self.chunk.code[self.ip..self.ip + 4];
                    self.ip += 4;
                    let sized_bytes = bytes.try_into().unwrap();
                    let index = u32::from_be_bytes(sized_bytes);
                    let key = self.chunk.constants[index as usize].as_str().to_string();
                    let value = self.pop();
                    self.globals.insert(key, value);
                }
                OpCode::GetGlobal => {
                    let bytes = &self.chunk.code[self.ip..self.ip + 4];
                    self.ip += 4;
                    let sized_bytes = bytes.try_into().unwrap();
                    let index = u32::from_be_bytes(sized_bytes);
                    let key = &self.chunk.constants[index as usize];
                    if let Some(v) = self.globals.get(key.as_str()) {
                        self.push(v.clone());
                    } else {
                        self.runtime_error(&format!("Undefined variable '{}'.", key.as_str()));
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::SetGlobal => {
                    let bytes = &self.chunk.code[self.ip..self.ip + 4];
                    self.ip += 4;
                    let sized_bytes = bytes.try_into().unwrap();
                    let index = u32::from_be_bytes(sized_bytes);
                    let key = self.chunk.constants[index as usize].as_str().to_string();
                    let value = self.peek(0); // todo use peek instead of pop: the value remains on the stack
                    if self.globals.get(key.as_str()).is_some() {
                        self.globals.insert(key, value.clone());
                    } else {
                        self.runtime_error(&format!("Undefined variable '{}'.", key.as_str()));
                        return InterpretResult::RuntimeError;
                    }
                }
            }
        }
    }

    fn concatenate(&mut self) {
        let b = self.pop();
        let mut a = self.pop().as_str().to_string();
        a.push_str(b.as_str());
        self.push(Value::string(&a));
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
            globals: HashMap::new(),
        };
        vm.run();
    } else {
    }
}
