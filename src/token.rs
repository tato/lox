use crate::value::LoxValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    lexeme: Vec<char>,
    pub literal: LoxValue,
    pub line: usize,
}
impl Token {
    pub fn new(kind: TokenKind, lexeme: Vec<char>, literal: LoxValue, line: usize) -> Self {
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
