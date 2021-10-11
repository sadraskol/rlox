use std::convert::TryInto;

pub type Value = f64;

pub enum OpCode {
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


pub struct Chunk {
    pub code: Vec<u8>,
    lines: Vec<usize>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: vec![],
            constants: vec![],
            lines: vec![],
        }
    }

    pub fn write_chunk(&mut self, code: OpCode, line: usize) {
        self.code.push(code.into());
        self.lines.push(line);
    }

    pub fn write_index(&mut self, index: u32, line: usize) {
        for b in index.to_be_bytes() {
            self.code.push(b);
            self.lines.push(line);
        }
    }

    pub fn add_constant(&mut self, constant: Value) -> u32 {
        if self.constants.len() >= u32::MAX as usize {
            panic!("cannot have more than {} constants.", u32::MAX);
        }
        self.constants.push(constant);
        (self.constants.len() - 1) as u32
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines[offset]);
        }
        match self.code[offset].into() {
            OpCode::OpReturn => println!("OP_RETURN"),
            OpCode::OpConstant => {
                let bytes = &self.code[offset + 1..offset + 5];
                let sized_bytes = bytes.try_into().unwrap();
                let index = u32::from_be_bytes(sized_bytes);
                println!(
                    "OP_CONSTANT    {} '{}'",
                    index, self.constants[index as usize]
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

}