use std::{any::Any};

#[derive(Debug, Clone, Copy)]
pub enum TokenKind {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

pub trait LiteralValue: std::fmt::Display + std::fmt::Debug {
}
impl LiteralValue for i64 { }
impl LiteralValue for f64 { }
impl LiteralValue for String { }

#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
    lexeme: Vec<char>,
    // https://www.reddit.com/r/rust/comments/5xm71l/extracting_original_type_from_boxany/
    // https://doc.rust-lang.org/std/any/index.html
    literal: Option<Box<dyn LiteralValue>>,
    line: usize,
}
impl Token {
    pub fn new(
        kind: TokenKind,
        lexeme: Vec<char>,
        literal: Option<Box<dyn LiteralValue>>,
        line: usize,
    ) -> Self {
        Self {
            kind,
            lexeme,
            literal,
            line,
        }
    }

    pub fn lexeme(&self) -> String {
        self.lexeme.iter().collect()
    }
}
