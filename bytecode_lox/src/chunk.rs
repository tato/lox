use lox_proc_macros::EnumVariantCount;

use crate::value::Value;

#[derive(Copy, Clone, EnumVariantCount)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Return,
}

impl OpCode {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
    pub fn from_u8(byte: u8) -> Option<Self> {
        if byte as usize >= Self::len() {
            None
        } else {
            unsafe { std::mem::transmute(byte) }
        }
    }
}

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self { code: vec![], constants: vec![], lines: vec![] }
    }
    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}