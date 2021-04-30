use lazy_static::lazy_static;
use lox_proc_macros::U8Enum;

#[cfg(feature = "debug_print_code")]
use crate::debug::disassemble_chunk;
use crate::{
    chunk::{Chunk, OpCode},
    error::{CompileError, ErrorInfo},
    scanner::{Scanner, Token, TokenKind},
    value::Value,
};

pub struct Compiler<'source> {
    chunk: Chunk,
    parser: Parser<'source>,
}

impl<'source> Compiler<'source> {
    pub fn compile(source: String) -> Result<Chunk, CompileError> {
        let scanner = Scanner::new(&source);

        let mut compiler = Compiler {
            chunk: Chunk::new(),
            parser: Parser::new(&scanner),
        };

        compiler.expression();
        compiler
            .parser
            .consume(TokenKind::Eof, "Expect end of expression.");
        compiler.end();

        Ok(compiler.chunk)
    }

    fn emit_byte(&mut self, byte: u8) {
        self.chunk.write(byte, self.parser.previous.line);
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::Constant.as_u8(), constant);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.chunk.add_constant(value);
        if constant > u8::MAX as usize {
            todo!("Too many constants in one chunk.");
            // return 0;
        }
        constant as u8
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.parser.advance();
        let prefix_rule = get_rule(self.parser.previous.kind).prefix;
        if let Some(prefix_rule) = prefix_rule {
            prefix_rule(self);
        } else {
            todo!("Expect expression.");
            // return;
        }

        while precedence.as_u8() <= get_rule(self.parser.current.kind).precedence.as_u8() {
            self.parser.advance();
            let infix_rule = get_rule(self.parser.previous.kind).infix;
            (infix_rule.unwrap())(self);
        }
    }

    fn end(&mut self) {
        self.emit_byte(OpCode::Return.as_u8());
        #[cfg(feature = "debug_print_code")]
        {
            if
            /* !self.parser.had_error */
            true {
                disassemble_chunk(&self.chunk, "code");
            }
        }
    }
}

fn grouping(compiler: &mut Compiler) {
    compiler.expression();
    compiler
        .parser
        .consume(TokenKind::RightParen, "Expect ')' after expression.");
}

fn number(compiler: &mut Compiler) {
    let value: Value = compiler.parser.previous.lexeme.parse().unwrap();
    compiler.emit_constant(value);
}

fn unary(compiler: &mut Compiler) {
    let operator_kind = compiler.parser.previous.kind;
    compiler.parse_precedence(Precedence::Unary);
    match operator_kind {
        TokenKind::Minus => compiler.emit_byte(OpCode::Negate.as_u8()),
        any => unreachable!("Can't parse operator kind '{:?}' as unary.", any),
    }
}

fn binary(compiler: &mut Compiler) {
    let operator_kind = compiler.parser.previous.kind;
    let rule = get_rule(operator_kind);
    compiler.parse_precedence(Precedence::from_u8(rule.precedence.as_u8() + 1).unwrap());

    match operator_kind {
        TokenKind::Plus => compiler.emit_byte(OpCode::Add.as_u8()),
        TokenKind::Minus => compiler.emit_byte(OpCode::Subtract.as_u8()),
        TokenKind::Star => compiler.emit_byte(OpCode::Multiply.as_u8()),
        TokenKind::Slash => compiler.emit_byte(OpCode::Divide.as_u8()),
        any => unreachable!("Can't parse operator kind '{:?}' as binary.", any),
    }
}

struct Parser<'source> {
    scanner: &'source Scanner<'source>,
    current: Token<'source>,
    previous: Token<'source>,
    panic_mode: bool,
}

impl<'source> Parser<'source> {
    pub fn new(scanner: &'source Scanner<'source>) -> Self {
        let token = scanner.scan();
        Self {
            scanner,
            current: token.clone(),
            previous: token,
            panic_mode: false,
        }
    }
    pub fn advance(&mut self) {
        self.previous = self.current.clone();
        loop {
            self.current = self.scanner.scan();
            if self.current.kind != TokenKind::Error {
                break;
            }
            self.panic_mode = true;
            eprintln!(
                "{}",
                CompileError::ScanError(ErrorInfo::error(&self.current, ""))
            )
        }
    }
    pub fn consume(&mut self, kind: TokenKind, message: &str) {
        if self.current.kind == kind {
            self.advance();
            return;
        }

        self.panic_mode = true;
        eprintln!(
            "{}",
            CompileError::ParseError(ErrorInfo::error(&self.current, message))
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, U8Enum)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

#[derive(Clone)]
struct ParseRule {
    prefix: Option<fn(&mut Compiler)>,
    infix: Option<fn(&mut Compiler)>,
    precedence: Precedence,
}

lazy_static! {
    static ref RULES: &'static [ParseRule] = {
        let mut rules = vec![None; TokenKind::COUNT];

        macro_rules! rule {
            ($kind:ident, $prefix:expr, $infix:expr, $precedence:ident) => {
                rules[TokenKind::$kind.as_u8() as usize] = Some(ParseRule {
                    prefix: $prefix,
                    infix: $infix,
                    precedence: Precedence::$precedence,
                });
            };
        }

        rule!(LeftParen, Some(grouping), None, None);
        rule!(RightParen, None, None, None);
        rule!(LeftBrace, None, None, None);
        rule!(RightBrace, None, None, None);
        rule!(Comma, None, None, None);
        rule!(Dot, None, None, None);
        rule!(Minus, Some(unary), Some(binary), Term);
        rule!(Plus, None, Some(binary), Term);
        rule!(Semicolon, None, None, None);
        rule!(Slash, None, Some(binary), Factor);
        rule!(Star, None, Some(binary), Factor);
        rule!(Bang, None, None, None);
        rule!(BangEqual, None, None, None);
        rule!(Equal, None, None, None);
        rule!(EqualEqual, None, None, None);
        rule!(Greater, None, None, None);
        rule!(GreaterEqual, None, None, None);
        rule!(Less, None, None, None);
        rule!(LessEqual, None, None, None);
        rule!(Identifier, None, None, None);
        rule!(String, None, None, None);
        rule!(Number, Some(number), None, None);
        rule!(And, None, None, None);
        rule!(Class, None, None, None);
        rule!(Else, None, None, None);
        rule!(False, None, None, None);
        rule!(For, None, None, None);
        rule!(Fun, None, None, None);
        rule!(If, None, None, None);
        rule!(Nil, None, None, None);
        rule!(Or, None, None, None);
        rule!(Print, None, None, None);
        rule!(Return, None, None, None);
        rule!(LeftBrace, None, None, None);
        rule!(Super, None, None, None);
        rule!(This, None, None, None);
        rule!(True, None, None, None);
        rule!(Var, None, None, None);
        rule!(While, None, None, None);
        rule!(Error, None, None, None);
        rule!(Eof, None, None, None);

        rules
            .into_iter()
            .map(Option::unwrap)
            .collect::<Vec<_>>()
            .leak()
    };
}

fn get_rule(kind: TokenKind) -> &'static ParseRule {
    &RULES[kind.as_u8() as usize]
}

// TODO! To really understand the parser, you need to see how execution threads
// through the interesting parsing functions—parsePrecedence() and the parser
// functions stored in the table. Take this (strange) expression:
//
//     (-1 + 2) * 3 - -4
//
// Write a trace of how those functions are called. Show the order they are
// called, which calls which, and the arguments passed to them.
