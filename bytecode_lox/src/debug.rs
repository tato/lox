use crate::chunk::{Chunk, OpCode};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);
    let mut offset = 0;
    while offset < chunk.code.len() {
        offset = disassemble_instruction(chunk, offset);
    }
}

fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);

    if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
        print!("   | ");
    } else {
        print!("{:4} ", chunk.lines[offset]);
    }

    let instruction = chunk.code[offset];
    match OpCode::from_u8(instruction) {
        Some(OpCode::Return) => simple_instruction("OP_RETURN", offset),
        Some(OpCode::Constant) => constant_instruction("OP_CONSTANT", chunk, offset),
        None => {
            println!("Unknown opcode {}", instruction);
            offset + 1
        },
    }
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{}", name);
    offset + 1
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.code[offset + 1];
    println!("{:-16} {:4} '{}'", name, constant, chunk.constants[constant as usize]);
    offset + 2
}