use std::{usize, vec};

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
    stack: Vec<Value>,
    chunk: &'chunk Chunk,
    ip: usize,
}

impl<'chunk> VM<'chunk> {
    pub fn new(chunk: &'chunk Chunk) -> Self {
        VM {
            chunk,
            ip: 0,
            stack: vec![],
        }
    }

    fn _reset_stack(&mut self) {
        self.stack.clear();
    }
    fn push(&mut self, value: Value) {
        if self.stack.len() >= STACK_MAX {
            panic!("Stack has reached maximum size!");
        }
        self.stack.push(value);
    }
    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap_or_default()
    }
    fn peek(&mut self, distance: usize) -> Value {
        self.stack[self.stack.len() - 1 - distance].clone()
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
                    self.chunk.constants[read_byte!() as usize].clone()
                };
            }
            macro_rules! binary_op {
                ($wrap:ident, $op:tt) => {{
                    match (self.peek(0), self.peek(1)) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.pop();
                            self.pop();
                            self.push(Value::$wrap(a $op b));
                        }
                        (_a, _b) => {
                            runtime_error!("Operands must be numbers.");
                            return Err(RuntimeError::OperandMustBeNumber("idk".to_string(), Value::Nil).into())
                        }
                    }
                }};
            }

            macro_rules! runtime_error {
                ($args:tt) => {{
                    eprint!("[line {}] ", self.chunk.get_line(self.ip));
                    eprintln!($args);
                }};
            }

            let opcode = read_byte!();
            let instruction = OpCode::from_u8(opcode).ok_or(RuntimeError::InvalidOpcode(opcode))?;

            match instruction {
                OpCode::Constant => {
                    let constant = read_constant!();
                    self.push(constant);
                }
                OpCode::Nil => self.push(Value::Nil),
                OpCode::False => self.push(Value::Bool(false)),
                OpCode::True => self.push(Value::Bool(true)),
                OpCode::Equal => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(Value::Bool(a.equals(&b)));
                },
                OpCode::NotEqual => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(Value::Bool(!a.equals(&b)));
                },
                OpCode::Greater => binary_op!(Bool, >),
                OpCode::GreaterEqual => binary_op!(Bool, >=),
                OpCode::Less => binary_op!(Bool, <),
                OpCode::LessEqual => binary_op!(Bool, <=),
                OpCode::Add => binary_op!(Number, +),
                OpCode::Subtract => binary_op!(Number, -),
                OpCode::Multiply => binary_op!(Number, *),
                OpCode::Divide => binary_op!(Number, /),
                OpCode::Not => {
                    let val = self.pop().is_falsey();
                    self.push(Value::Bool(val));
                }
                OpCode::Negate => {
                    if let Value::Number(number) = self.peek(0) {
                        self.pop();
                        self.push(Value::Number(-number))
                    } else {
                        runtime_error!("Operand must be a number.");
                        return Err(RuntimeError::OperandMustBeNumber(
                            "unary negation".to_string(),
                            self.peek(0),
                        )
                        .into());
                    }
                }
                OpCode::Return => {
                    println!("{}", self.pop());
                    return Ok(());
                }
            }
        }
    }

    pub fn interpret(source: String) -> Result<(), InterpretError> {
        let chunk = Compiler::compile(source)?;
        let mut vm = VM::new(&chunk);
        vm.run()
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
