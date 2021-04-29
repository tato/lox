use lox_proc_macros::U8Enum;

use crate::value::Value;

// Because OP_CONSTANT uses only a single byte for its operand, a chunk may
// only contain up to 256 different constants. That’s small enough that people
// writing real-world code will hit that limit. We could use two or more bytes
// to store the operand, but that makes every constant instruction take up
// more space. Most chunks won’t need that many unique constants, so that
// wastes space and sacrifices some locality in the common case to support the
// rare case.
//
// To balance those two competing aims, many instruction sets feature multiple
// instructions that perform the same operation but with operands of different
// sizes. Leave our existing one-byte OP_CONSTANT instruction alone, and define
// a second OP_CONSTANT_LONG instruction. It stores the operand as a 24-bit
// number, which should be plenty.
//
// Implement this function:
//
//     void writeConstant(Chunk* chunk, Value value, int line) {
//         // Implement me...
//     }
//
// It adds value to chunk’s constant array and then writes an appropriate
// instruction to load the constant. Also add support to the disassembler for
// OP_CONSTANT_LONG instructions.
//
// Defining two instructions seems to be the best of both worlds. What
// sacrifices, if any, does it force on us?

#[derive(Copy, Clone, U8Enum)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}

struct LineInfo {
    count: u32,
    line: u32, // I hope nobody has more than 4.294.967.295 lines in a source file
}

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    // line information is stored using run-length encoding
    lines: Vec<LineInfo>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            constants: vec![],
            lines: vec![],
        }
    }
    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);

        let line = line as u32; // TODO! checked

        if self.lines.last().map(|it| it.line == line).unwrap_or(false) {
            let len = self.lines.len();
            self.lines[len - 1].count += 1;
        } else {
            self.lines.push(LineInfo {
                count: 1,
                line: line,
            });
        }
    }
    pub fn get_line(&self, offset: usize) -> u32 {
        let mut i = 0;
        for line in &self.lines {
            i += line.count as usize;
            if offset < i {
                return line.line;
            }
        }
        u32::MAX
    }
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}
