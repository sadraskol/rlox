use std::convert::TryInto;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub arity: u32,
    pub chunk: Chunk,
    pub name: String,
}

impl Function {
    pub fn new(arity: u32, name: &str) -> Self {
        Function {
            arity,
            name: name.to_string(),
            chunk: Chunk::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Closure {
    pub function: Rc<Function>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    Str(String),
    Closure(Closure),
}

impl Object {
    pub fn print(&self) -> String {
        match self {
            Object::Str(s) => s.to_string(),
            Object::Closure(Closure { function }) => {
                if function.name == "<script>" {
                    "<script>".to_string()
                } else {
                    format!("<fn {}>", function.name)
                }
            }
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

impl Value {
    pub fn from_number(n: f64) -> Self {
        Value::Number(n)
    }
    pub fn from_bool(b: bool) -> Self {
        Value::Bool(b)
    }
    pub fn string(s: &str) -> Self {
        let string = Object::Str(s.to_string());
        Value::Obj(Box::new(string))
    }
    pub fn closure(function: Rc<Function>) -> Self {
        let closure = Object::Closure(Closure { function });
        Value::Obj(Box::new(closure))
    }
    pub fn nil() -> Self {
        Value::Nil
    }

    pub fn is_string(&self) -> bool {
        if let Value::Obj(o) = self {
            matches!(&**o, Object::Str { .. })
        } else {
            false
        }
    }
    pub fn is_closure(&self) -> bool {
        if let Value::Obj(o) = self {
            matches!(&**o, Object::Closure(_))
        } else {
            false
        }
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
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
            if let Object::Str(s) = &**o {
                s
            } else {
                panic!("not a string");
            }
        } else {
            panic!("not an object");
        }
    }

    pub fn as_function(&self) -> Rc<Function> {
        if let Value::Obj(o) = self {
            if let Object::Closure(c) = &**o {
                c.function.clone()
            } else {
                panic!("not a string");
            }
        } else {
            panic!("not an object");
        }
    }

    pub fn as_closure(&self) -> Closure {
        if let Value::Obj(o) = self {
            if let Object::Closure(c) = &**o {
                c.clone()
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
    GetLocal,
    SetLocal,
    JumpIfFalse,
    Jump,
    Loop,
    Call,
    Closure,
    Debug,
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
            14 => OpCode::JumpIfFalse,
            15 => OpCode::GetLocal,
            16 => OpCode::SetLocal,
            17 => OpCode::Jump,
            18 => OpCode::Loop,
            19 => OpCode::Call,
            20 => OpCode::Closure,
            255 => OpCode::Debug,
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
            OpCode::JumpIfFalse => 14,
            OpCode::GetLocal => 15,
            OpCode::SetLocal => 16,
            OpCode::Jump => 17,
            OpCode::Loop => 18,
            OpCode::Call => 19,
            OpCode::Closure => 20,
            OpCode::Debug => 255,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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
        if self.code.len() >= u32::MAX as usize {
            panic!("Source code too long!");
        }
        self.code.push(code.into());
        self.lines.push(line);
    }

    pub fn write_u32(&mut self, index: u32, line: usize) {
        for b in index.to_be_bytes() {
            if self.code.len() >= u32::MAX as usize {
                panic!("Source code too long!");
            }
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

    pub fn size(&self) -> u32 {
        self.code.len() as u32
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
            OpCode::Closure => {
                let bytes = &self.code[offset + 1..offset + 5];
                let sized_bytes = bytes.try_into().unwrap();
                let index = u32::from_be_bytes(sized_bytes);
                println!(
                    "OP_CLOSURE       {} '{:?}'",
                    index, self.constants[index as usize]
                );
                return offset + 5;
            }
            OpCode::Call => {
                let bytes = &self.code[offset + 1..offset + 5];
                let sized_bytes = bytes.try_into().unwrap();
                let args_c = u32::from_be_bytes(sized_bytes);
                println!("OP_CALL      {}", args_c);
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
            OpCode::Debug => println!("OP_DEBUG"),
            OpCode::JumpIfFalse => {
                let bytes = &self.code[offset + 1..offset + 5];
                let sized_bytes = bytes.try_into().unwrap();
                let index = u32::from_be_bytes(sized_bytes);
                println!("OP_JUMP_IF_FALSE {}", index);
                return offset + 5;
            }
            OpCode::Jump => {
                let bytes = &self.code[offset + 1..offset + 5];
                let sized_bytes = bytes.try_into().unwrap();
                let index = u32::from_be_bytes(sized_bytes);
                println!("OP_JUMP          {}", index);
                return offset + 5;
            }
            OpCode::Loop => {
                let bytes = &self.code[offset + 1..offset + 5];
                let sized_bytes = bytes.try_into().unwrap();
                let index = u32::from_be_bytes(sized_bytes);
                println!("OP_LOOP          {}", index);
                return offset + 5;
            }
            OpCode::GetLocal => {
                let bytes = &self.code[offset + 1..offset + 5];
                let sized_bytes = bytes.try_into().unwrap();
                let index = u32::from_be_bytes(sized_bytes);
                println!("OP_GET_LOCAL     {}", index);
                return offset + 5;
            }
            OpCode::SetLocal => {
                let bytes = &self.code[offset + 1..offset + 5];
                let sized_bytes = bytes.try_into().unwrap();
                let index = u32::from_be_bytes(sized_bytes);
                println!("OP_SET_LOCAL     {}", index);
                return offset + 5;
            }
        }
        offset + 1
    }
}
