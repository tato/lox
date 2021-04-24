use error::ErrorReporter;
use interpreter::Interpreter;
use parser::Parser;
use resolver::Resolver;
use scanner::Scanner;

mod ast;
mod environment;
mod error;
mod interpreter;
mod parser;
mod resolver;
mod scanner;
mod token;
mod value;

struct Lox {
    _reporter: ErrorReporter,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            _reporter: ErrorReporter::new(),
        }
    }

    pub fn run(&mut self, source: String) -> anyhow::Result<()> {
        let tokens = Scanner::new(source).scan_tokens()?;
        let statements = Parser::new(tokens).parse()?;

        let mut interpreter = Interpreter::new();
        let mut resolver = Resolver::new(&mut interpreter);
        resolver.resolve(&statements);
        interpreter.interpret(&statements);

        Ok(())
    }

    pub fn run_file(&mut self, path: &str) -> anyhow::Result<()> {
        let bytes = std::fs::read(path)?;
        self.run(std::str::from_utf8(&bytes)?.into())
    }

    pub fn run_prompt(&mut self) -> anyhow::Result<()> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        use std::io::{BufRead, Write};
        loop {
            print!("> ");
            stdout.flush()?;
            let mut line = String::new();
            let mut reader = stdin.lock();
            if reader.read_line(&mut line)? == 0 {
                break;
            }
            if let Err(error) = self.run(line) {
                println!("{}", error);
            }
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() > 2 {
        println!("Usage: lox [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        let mut lox = Lox::new();
        lox.run_file(&args[1])?;
    } else {
        let mut lox = Lox::new();
        lox.run_prompt()?;
    }
    Ok(())
}
