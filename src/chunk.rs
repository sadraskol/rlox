use std::convert::TryInto;

#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    Str {
        s: String
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Str {s} => f.write_str(s),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Obj(Box<Object>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => f.write_str("nil"),
            Value::Bool(true) => f.write_str("true"),
            Value::Bool(false) => f.write_str("false"),
            Value::Number(n) => n.fmt(f),
            Value::Obj(obj) => obj.fmt(f),
        }
        
    }
}

impl Value {
    pub fn from_number(n: f64) -> Self {
        Value::Number(n)
    }
    pub fn from_bool(b: bool) -> Self {
        Value::Bool(b)
    }
    pub fn string(s: &str) -> Self {
        let string = Object::Str {
            s: s.to_string(),
        };
        Value::Obj(Box::new(string))
    }
    pub fn nil() -> Self {
        Value::Nil
    }

    pub fn is_string(&self) -> bool {
        if let Value::Obj(o) = self {
            if let Object::Str {..} = **o {
                true 
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn is_bool(&self) -> bool {
        if let Value::Bool(_) = self {
            true 
        } else {
            false
        }
    }
    pub fn is_nil(&self) -> bool {
        self == &Value::Nil
    }
    pub fn is_number(&self) -> bool {
        if let Value::Number(_) = self {
            true 
        } else {
            false
        }
    }

    pub fn as_number(&self) -> f64 {
        if let Value::Number(n) = self {
            *n
        } else {
            panic!("not a number");
        }
    }

    pub fn as_bool(&self) -> bool {
        if let Value::Bool(b) = self {
            *b
        } else {
            panic!("not a bool");
        }
    }

    pub fn as_str(&self) -> &str {
        if let Value::Obj(b) = self {
            if let Object::Str { s } = &**b {
                &s
            } else {
                panic!("not a string");
            }
        } else {
            panic!("not an obj");
        }
    }

}

pub enum OpCode {
    OpReturn,
    OpConstant,
    OpDivide,
    OpAdd,
    OpNegate,
    OpMultiply,
    OpSubstract,
    OpNot,
    OpEqual,
    OpGreater,
    OpLess,
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
            7 => OpCode::OpNot,
            8 => OpCode::OpEqual,
            9 => OpCode::OpGreater,
            10 => OpCode::OpLess,
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
            OpCode::OpNot => 7,
            OpCode::OpEqual => 8,
            OpCode::OpGreater => 9,
            OpCode::OpLess => 10,
        }
    }
}

#[derive(Clone)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub lines: Vec<usize>,
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
                    "OP_CONSTANT    {} '{:?}'",
                    index, self.constants[index as usize]
                );
                return offset + 5;
            }
            OpCode::OpDivide => println!("OP_DIVIDE"),
            OpCode::OpAdd => println!("OP_ADD"),
            OpCode::OpNegate => println!("OP_NEGATE"),
            OpCode::OpMultiply => println!("OP_MULTIPLY"),
            OpCode::OpSubstract => println!("OP_SUBSTRACT"),
            OpCode::OpNot => println!("OP_NOT"),
            OpCode::OpEqual => println!("OP_EQUAL"),
            OpCode::OpGreater => println!("OP_GREATER"),
            OpCode::OpLess => println!("OP_LESS"),
        }
        offset + 1
    }

}
