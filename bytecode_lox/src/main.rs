use chunk::{Chunk, OpCode};

use vm::VM;

mod chunk;
#[cfg(feature = "debug_trace_execution")]
mod debug;
mod error;
mod value;
mod vm;

fn main() {
    let mut vm = VM::new();

    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant.as_u8(), 123);
    chunk.write(constant as u8, 123);

    let constant = chunk.add_constant(3.4);
    chunk.write(OpCode::Constant.as_u8(), 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::Add.as_u8(), 123);

    let constant = chunk.add_constant(5.6);
    chunk.write(OpCode::Constant.as_u8(), 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::Divide.as_u8(), 123);
    chunk.write(OpCode::Negate.as_u8(), 123);
    chunk.write(OpCode::Return.as_u8(), 123);

    vm.interpret(&chunk).unwrap();
}
