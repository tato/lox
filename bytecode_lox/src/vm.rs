use std::usize;

use crate::{
    chunk::{Chunk, OpCode},
    compiler::Compiler,
    error::{InterpretError, RuntimeError},
    value::Value,
};

#[cfg(feature = "debug_trace_execution")]
use crate::debug::disassemble_instruction;

const STACK_MAX: usize = 256;
pub struct VM<'chunk> {
    stack: Box<[Value; STACK_MAX]>,
    stack_top: usize,
    chunk: &'chunk Chunk,
    ip: usize,
}

impl<'chunk> VM<'chunk> {
    pub fn new(chunk: &'chunk Chunk) -> Self {
        VM {
            chunk,
            ip: 0,
            stack: [Default::default(); STACK_MAX].into(),
            stack_top: 0,
        }
    }

    fn _reset_stack(&mut self) {
        self.stack_top = 0;
    }
    fn push(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }
    fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top]
    }

    fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            #[cfg(feature = "debug_trace_execution")]
            {
                print!("          ");
                for i in 0..self.stack_top {
                    print!("[ {} ]", self.stack[i]);
                }
                println!("");
                disassemble_instruction(self.chunk, self.ip); // TODO! VERY slow!! makes the loop O(n^2)!
            }

            macro_rules! read_byte {
                () => {{
                    self.ip += 1;
                    self.chunk.code[self.ip - 1]
                }};
            }
            macro_rules! read_constant {
                () => {
                    self.chunk.constants[read_byte!() as usize]
                };
            }
            macro_rules! binary_op {
                ($op:tt) => {{
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a $op b);
                }};
            }

            let opcode = read_byte!();
            let instruction = OpCode::from_u8(opcode).ok_or(RuntimeError::InvalidOpcode(opcode))?;

            match instruction {
                OpCode::Constant => {
                    let constant = read_constant!();
                    self.push(constant);
                }
                OpCode::Add => binary_op!(+),
                OpCode::Subtract => binary_op!(-),
                OpCode::Multiply => binary_op!(*),
                OpCode::Divide => binary_op!(/),
                OpCode::Negate => {
                    let val = -self.pop();
                    self.push(val);
                }
                OpCode::Return => {
                    println!("{}", self.pop());
                    return Ok(());
                }
            }
        }
    }

    pub fn interpret(source: &str) -> Result<(), InterpretError> {
        // let mut vm = VM::new(chunk);
        // vm.run()
        Compiler::compile(source);
        Ok(())
    }
}

// TODO!
// Our VM’s stack has a fixed size, and we don’t check if pushing a value
// overflows it. This means the wrong series of instructions could cause
// our interpreter to crash or go into undefined behavior. Avoid that by
// dynamically growing the stack as needed.
//
// What are the costs and benefits of doing so?

// TODO!
// To interpret OP_NEGATE, we pop the operand, negate the value, and then push
// the result. That’s a simple implementation, but it increments and
// decrements stackTop unnecessarily, since the stack ends up the same height
// in the end. It might be faster to simply negate the value in place on the
// stack and leave stackTop alone. Try that and see if you can measure a
// performance difference.
//
// Are there other instructions where you can do a similar optimization?
