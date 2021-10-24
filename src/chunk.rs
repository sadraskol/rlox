use std::convert::TryInto;

#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    Str { s: String },
}

impl Object {
    pub fn print(&self) -> String {
        let Object::Str { s } = self;
        s.to_string()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Obj(Box<Object>),
}

impl Value {
    pub fn from_number(n: f64) -> Self {
        Value::Number(n)
    }
    pub fn from_bool(b: bool) -> Self {
        Value::Bool(b)
    }
    pub fn string(s: &str) -> Self {
        let string = Object::Str { s: s.to_string() };
        Value::Obj(Box::new(string))
    }
    pub fn nil() -> Self {
        Value::Nil
    }

    pub fn is_string(&self) -> bool {
        if let Value::Obj(o) = self {
            if let Object::Str { .. } = &**o {
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
        if let Value::Obj(o) = self {
            if let Object::Str { s } = &**o {
                &s
            } else {
                panic!("not a string");
            }
        } else {
            panic!("not an object");
        }
    }

    pub fn print(&self) -> String {
        match self {
            Value::Nil => "nil".to_string(),
            Value::Bool(true) => "true".to_string(),
            Value::Bool(false) => "false".to_string(),
            Value::Number(f) => f.to_string(),
            Value::Obj(o) => o.print(),
        }
    }
}

pub enum OpCode {
    Return,
    Constant,
    Divide,
    Add,
    Negate,
    Multiply,
    Substract,
    Not,
    Equal,
    Greater,
    Less,
    Print,
    Nil,
    Pop,
    DefineGlobal,
    GetGlobal,
}

impl From<u8> for OpCode {
    fn from(b: u8) -> Self {
        match b {
            0 => OpCode::Return,
            1 => OpCode::Constant,
            2 => OpCode::Divide,
            3 => OpCode::Add,
            4 => OpCode::Negate,
            5 => OpCode::Multiply,
            6 => OpCode::Substract,
            7 => OpCode::Not,
            8 => OpCode::Equal,
            9 => OpCode::Greater,
            10 => OpCode::Less,
            11 => OpCode::Print,
            12 => OpCode::Nil,
            13 => OpCode::Pop,
            14 => OpCode::DefineGlobal,
            15 => OpCode::GetGlobal,
            _ => panic!("unexpected op code"),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(b: OpCode) -> Self {
        match b {
            OpCode::Return => 0,
            OpCode::Constant => 1,
            OpCode::Divide => 2,
            OpCode::Add => 3,
            OpCode::Negate => 4,
            OpCode::Multiply => 5,
            OpCode::Substract => 6,
            OpCode::Not => 7,
            OpCode::Equal => 8,
            OpCode::Greater => 9,
            OpCode::Less => 10,
            OpCode::Print => 11,
            OpCode::Nil => 12,
            OpCode::Pop => 13,
            OpCode::DefineGlobal => 14,
            OpCode::GetGlobal => 15,
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
            OpCode::Return => println!("OP_RETURN"),
            OpCode::Constant => {
                let bytes = &self.code[offset + 1..offset + 5];
                let sized_bytes = bytes.try_into().unwrap();
                let index = u32::from_be_bytes(sized_bytes);
                println!(
                    "OP_CONSTANT      {} '{:?}'",
                    index, self.constants[index as usize]
                );
                return offset + 5;
            }
            OpCode::Divide => println!("OP_DIVIDE"),
            OpCode::Add => println!("OP_ADD"),
            OpCode::Negate => println!("OP_NEGATE"),
            OpCode::Multiply => println!("OP_MULTIPLY"),
            OpCode::Substract => println!("OP_SUBSTRACT"),
            OpCode::Not => println!("OP_NOT"),
            OpCode::Equal => println!("OP_EQUAL"),
            OpCode::Greater => println!("OP_GREATER"),
            OpCode::Less => println!("OP_LESS"),
            OpCode::Print => println!("OP_PRINT"),
            OpCode::Nil => println!("OP_NIL"),
            OpCode::Pop => println!("OP_POP"),
            OpCode::DefineGlobal => {
                let bytes = &self.code[offset + 1..offset + 5];
                let sized_bytes = bytes.try_into().unwrap();
                let index = u32::from_be_bytes(sized_bytes);
                println!(
                    "OP_DEFINE_GLOBAL {} '{:?}'",
                    index, self.constants[index as usize]
                );
                return offset + 5;
            },
            OpCode::GetGlobal => {
                let bytes = &self.code[offset + 1..offset + 5];
                let sized_bytes = bytes.try_into().unwrap();
                let index = u32::from_be_bytes(sized_bytes);
                println!(
                    "OP_GET_GLOBAL    {} '{:?}'",
                    index, self.constants[index as usize]
                );
                return offset + 5;
            }
        }
        offset + 1
    }
}
