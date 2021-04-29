use crate::scanner::{Scanner, TokenKind};

pub struct Compiler {}

impl Compiler {
    pub fn compile(source: &str) {
        let source_chars = {
            let mut v: Vec<char> = source.chars().collect();
            v.push('\0');
            v
        };
        let scanner = Scanner::new(&source_chars);

        let mut line = 0;
        loop {
            let token = scanner.scan();
            if token.line != line {
                print!("{:4} ", token.line);
                line = token.line;
            } else {
                print!("   | ");
            }
            println!(
                "{:2} '{}'",
                token.kind.as_u8(),
                token.lexeme.iter().collect::<String>()
            );

            if token.kind == TokenKind::Eof {
                break;
            }
        }
    }
}
