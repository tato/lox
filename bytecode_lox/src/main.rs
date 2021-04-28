use bytemuck::bytes_of;
use chunk::{Chunk, OpCode};
use debug::disassemble_chunk;
use value::Value;

mod chunk;
mod debug;
mod value;

fn main() {
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(Value(1.2));
    chunk.write(OpCode::Constant.as_u8(), 123);
    chunk.write(constant as u8, 123);
    chunk.write(OpCode::Return.as_u8(), 123);
    disassemble_chunk(&chunk, "test chunk");
}
