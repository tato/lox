use crate::chunk::{Chunk, OpCode};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);
    let mut offset = 0;
    while offset < chunk.code.len() {
        offset = disassemble_instruction(chunk, offset);
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);

    if offset > 0 && chunk.get_line(offset) == chunk.get_line(offset - 1) {
        print!("   | ");
    } else {
        print!("{:4} ", chunk.get_line(offset));
    }

    let instruction = chunk.code[offset];
    match OpCode::from_u8(instruction) {
        Some(OpCode::Return) => simple_instruction("OP_RETURN", offset),
        Some(OpCode::Negate) => simple_instruction("OP_NEGATE", offset),
        Some(OpCode::Add) => simple_instruction("OP_ADD", offset),
        Some(OpCode::Subtract) => simple_instruction("OP_SUBTRACT", offset),
        Some(OpCode::Multiply) => simple_instruction("OP_MULTIPLY", offset),
        Some(OpCode::Divide) => simple_instruction("OP_DIVIDE", offset),
        Some(OpCode::Not) => simple_instruction("OP_NOT", offset),
        Some(OpCode::Constant) => constant_instruction("OP_CONSTANT", chunk, offset),
        Some(OpCode::Nil) => simple_instruction("OP_NIL", offset),
        Some(OpCode::False) => simple_instruction("OP_FALSE", offset),
        Some(OpCode::True) => simple_instruction("OP_TRUE", offset),
        Some(OpCode::Equal) => simple_instruction("OP_EQUAL", offset),
        Some(OpCode::NotEqual) => simple_instruction("OP_NOTEQUAL", offset),
        Some(OpCode::Greater) => simple_instruction("OP_GREATER", offset),
        Some(OpCode::GreaterEqual) => simple_instruction("OP_GREATEREQUAL", offset),
        Some(OpCode::Less) => simple_instruction("OP_LESS", offset),
        Some(OpCode::LessEqual) => simple_instruction("OP_LESSEQUAL", offset),
        None => {
            println!("Unknown opcode {}", instruction);
            offset + 1
        }
    }
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{}", name);
    offset + 1
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.code[offset + 1];
    println!(
        "{:-16} {:4} '{}'",
        name, constant, chunk.constants[constant as usize]
    );
    offset + 2
}
