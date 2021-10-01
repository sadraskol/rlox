use std::convert::TryInto;

mod compiler;

type Value = f64;

enum OpCode {
    OpReturn,
    OpConstant,
    OpDivide,
    OpAdd,
    OpNegate,
    OpMultiply,
    OpSubstract,
}

impl From<u8> for OpCode {
    fn from(b: u8) -> Self {
        match b {
            0 => OpCode::OpReturn,
            1 => OpCode::OpConstant,
            2 => OpCode::OpDivide,
            3 => OpCode::OpAdd,
            4 => OpCode::OpNegate,
            5 => OpCode::OpMultiply,
            6 => OpCode::OpSubstract,
            _ => panic!("unexpected op code"),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(b: OpCode) -> Self {
        match b {
            OpCode::OpReturn => 0,
            OpCode::OpConstant => 1,
            OpCode::OpDivide => 2,
            OpCode::OpAdd => 3,
            OpCode::OpNegate => 4,
            OpCode::OpMultiply => 5,
            OpCode::OpSubstract => 6,
        }
    }
}

struct VM<'a> {
    chunk: &'a Chunk,
    ip: usize,
    stack: Vec<Value>,
}

enum InterpretResult {
    InterpretOk,
}

impl<'a> VM<'a> {
    fn new(chunk: &'a Chunk) -> Self {
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
                    return InterpretResult::InterpretOk;
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
}

struct Chunk {
    code: Vec<u8>,
    lines: Vec<usize>,
    constants: Vec<Value>,
}

impl Chunk {
    fn new() -> Self {
        Chunk {
            code: vec![],
            constants: vec![],
            lines: vec![],
        }
    }

    fn write_chunk(&mut self, code: OpCode, line: usize) {
        self.code.push(code.into());
        self.lines.push(line);
    }

    fn write_index(&mut self, index: u32, line: usize) {
        for b in index.to_be_bytes() {
            self.code.push(b);
            self.lines.push(line);
        }
    }

    fn add_constant(&mut self, constant: Value) -> u32 {
        if self.constants.len() >= u32::MAX as usize {
            panic!("cannot have more than {} constants.", u32::MAX);
        }
        self.constants.push(constant);
        (self.constants.len() - 1) as u32
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

    disassemble_chunk(&chunks, "test chunk");

    let mut vm = VM::new(&chunks);
    vm.run();
}

fn disassemble_chunk(chunks: &Chunk, name: &str) {
    println!("== {} ==", name);
    let mut offset = 0;
    while offset < chunks.code.len() {
        offset = disassemble_instruction(chunks, offset);
    }
}

fn disassemble_instruction(chunks: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);
    if offset > 0 && chunks.lines[offset] == chunks.lines[offset - 1] {
        print!("   | ");
    } else {
        print!("{:4} ", chunks.lines[offset]);
    }
    match chunks.code[offset].into() {
        OpCode::OpReturn => println!("OP_RETURN"),
        OpCode::OpConstant => {
            let bytes = &chunks.code[offset + 1..offset + 5];
            let sized_bytes = bytes.try_into().unwrap();
            let index = u32::from_be_bytes(sized_bytes);
            println!(
                "OP_CONSTANT    {} '{}'",
                index, chunks.constants[index as usize]
            );
            return offset + 5;
        }
        OpCode::OpDivide => println!("OP_DIVIDE"),
        OpCode::OpAdd => println!("OP_ADD"),
        OpCode::OpNegate => println!("OP_NEGATE"),
        OpCode::OpMultiply => println!("OP_MULTIPLY"),
        OpCode::OpSubstract => println!("OP_SUBSTRACT"),
    }
    offset + 1
}
