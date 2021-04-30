use error::InterpretError;
use vm::VM;

mod chunk;
mod compiler;
#[cfg(any(feature = "debug_trace_execution", feature = "debug_print_code"))]
mod debug;
mod error;
mod iterator;
mod scanner;
mod value;
mod vm;

pub struct Lox {}

fn handle_interpret_error(error: &InterpretError) {
    match error {
        InterpretError::Compile(e) => {
            eprintln!("{}", e);
            std::process::exit(65);
        }
        InterpretError::Runtime(e) => {
            eprintln!("{}", e);
            std::process::exit(70);
        }
    }
}

impl Lox {
    pub fn run_file(path: &str) {
        let bytes = std::fs::read(path).unwrap();
        let result = VM::interpret(String::from_utf8(bytes).unwrap());
        result.as_ref().map_err(handle_interpret_error);
        result.unwrap();
    }

    pub fn run_prompt() {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        use std::io::{BufRead, Write};
        loop {
            print!("> ");
            stdout.flush().unwrap();
            let mut line = String::new();
            let mut reader = stdin.lock();
            if reader.read_line(&mut line).unwrap() == 0 {
                break;
            }
            if let Err(error) = VM::interpret(line) {
                handle_interpret_error(&error);
            }
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() > 2 {
        println!("Usage: lox [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        Lox::run_file(&args[1]);
    } else {
        Lox::run_prompt();
    }
}
