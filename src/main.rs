use crate::chunk::Function;
use crate::chunk::OpCode;
use crate::chunk::Value;
use crate::compiler::Parser;
use std::convert::TryInto;
use std::env::args;

mod chunk;
mod compiler;

#[derive(Debug)]
struct CallStack {
    function: Function,
    ip: usize,
    offset: usize,
}

struct VM {
    frames: Vec<CallStack>,
    stack: Vec<Value>,
}

enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

impl VM {
    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn push(&mut self, v: Value) {
        self.stack.push(v);
    }

    fn frame(&self) -> &CallStack {
        self.frames.last().unwrap()
    }

    fn frame_mut(&mut self) -> &mut CallStack {
        self.frames.last_mut().unwrap()
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            let instruction = self.frame().function.chunk.code[self.frame().ip];
            self.frame_mut().ip += 1;
            match instruction.into() {
                OpCode::Return => {
                    return InterpretResult::Ok;
                }
                OpCode::Constant => {
                    let bytes =
                        &self.frame().function.chunk.code[self.frame().ip..self.frame().ip + 4];
                    let sized_bytes = bytes.try_into().unwrap();
                    self.frame_mut().ip += 4;
                    let index = u32::from_be_bytes(sized_bytes);
                    let constant = (&self.frame().function.chunk.constants[index as usize]).clone();
                    self.stack.push(constant);
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
                    println!("{}", self.pop().print());
                }
                OpCode::Nil => {
                    self.push(Value::Nil);
                }
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::JumpIfFalse => {
                    let bytes =
                        &self.frame().function.chunk.code[self.frame().ip..self.frame().ip + 4];
                    let sized_bytes = bytes.try_into().unwrap();
                    self.frame_mut().ip += 4;
                    let jump = u32::from_be_bytes(sized_bytes);
                    if !self.peek(0).as_bool() {
                        self.frame_mut().ip += jump as usize;
                    }
                }
                OpCode::Jump => {
                    let bytes =
                        &self.frame().function.chunk.code[self.frame().ip..self.frame().ip + 4];
                    let sized_bytes = bytes.try_into().unwrap();
                    self.frame_mut().ip += 4;
                    let jump = u32::from_be_bytes(sized_bytes);
                    self.frame_mut().ip += jump as usize;
                }
                OpCode::Loop => {
                    let bytes =
                        &self.frame().function.chunk.code[self.frame().ip..self.frame().ip + 4];
                    let sized_bytes = bytes.try_into().unwrap();
                    self.frame_mut().ip += 4;
                    let jump = u32::from_be_bytes(sized_bytes);
                    self.frame_mut().ip -= jump as usize;
                }
                OpCode::GetLocal => {
                    let bytes =
                        &self.frame().function.chunk.code[self.frame().ip..self.frame().ip + 4];
                    let sized_bytes = bytes.try_into().unwrap();
                    self.frame_mut().ip += 4;
                    let index = u32::from_be_bytes(sized_bytes);
                    self.push(self.stack[self.frame().offset + index as usize].clone());
                }
                OpCode::SetLocal => {
                    let bytes =
                        &self.frame().function.chunk.code[self.frame().ip..self.frame().ip + 4];
                    let sized_bytes = bytes.try_into().unwrap();
                    self.frame_mut().ip += 4;
                    let index = u32::from_be_bytes(sized_bytes);
                    let offset = self.frame().offset;
                    let value = self.peek(0).clone();
                    self.stack[offset + index as usize] = value;
                }
                OpCode::Call => {
                    let bytes =
                        &self.frame().function.chunk.code[self.frame().ip..self.frame().ip + 4];
                    let sized_bytes = bytes.try_into().unwrap();
                    self.frame_mut().ip += 4;
                    let args_c = u32::from_be_bytes(sized_bytes);
                    if !self.call(args_c) {
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::Debug => {
                    for v in &self.stack {
                        print!("[{}] ", v.print());
                    }
                    println!();

                    println!("frame {:?}", self.frame());
                }
            }
        }
    }

    fn call(&mut self, argc: u32) -> bool {
        let f = self.peek(argc as usize);
        if f.is_function() {
            let function = f.as_function().clone();
            if function.arity != argc {
                self.runtime_error(&format!("Expected {} arguments but got {}.", function.arity, argc));
                return false;
            } else {
                self.frames.push(CallStack {
                    function,
                    ip: 0,
                    offset: self.stack.len() - argc as usize
                });
                true
            }
        } else {
            false
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
        for frame in self.frames.iter().rev() {
            let instruction = frame.ip - 1;
            eprintln!(
                "[line {}] in {}",
                frame.function.chunk.lines[instruction],
                frame.function.name
            );
        }
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
    let script = compiler.compile();

    if let Some(script) = script {
        let mut vm = VM {
            frames: vec![CallStack {
                function: script,
                offset: 0,
                ip: 0,
            }],
            stack: vec![],
        };
        vm.run();
    } else {
    }
}
