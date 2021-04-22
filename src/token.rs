use std::fmt::Display;

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
pub enum LiteralValue {
    Bool(bool),
    // Int(i64),
    Float(f64),
    Str(String),
    Nil,
}
impl Display for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralValue::Bool(x) => write!(f, "{}", x),
            // LiteralValue::Int(x) => write!(f, "{}", x),
            LiteralValue::Float(x) => write!(f, "{}", x),
            LiteralValue::Str(x) => write!(f, "{}", x),
            LiteralValue::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    lexeme: Vec<char>,
    pub literal: LiteralValue,
    pub line: usize,
}
impl Token {
    pub fn new(kind: TokenKind, lexeme: Vec<char>, literal: LiteralValue, line: usize) -> Self {
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
