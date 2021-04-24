use lazy_static::lazy_static;
use std::{collections::hash_map::HashMap, error::Error, fmt::Display};

use crate::token::{Token, TokenKind};
use crate::{value::LoxValue};

lazy_static! {
    static ref RESERVED_WORDS: HashMap<String, TokenKind> = {
        let mut m = HashMap::new();
        m.insert("and".into(), TokenKind::And);
        m.insert("class".into(), TokenKind::Class);
        m.insert("else".into(), TokenKind::Else);
        m.insert("false".into(), TokenKind::False);
        m.insert("for".into(), TokenKind::For);
        m.insert("fun".into(), TokenKind::Fun);
        m.insert("if".into(), TokenKind::If);
        m.insert("nil".into(), TokenKind::Nil);
        m.insert("or".into(), TokenKind::Or);
        m.insert("print".into(), TokenKind::Print);
        m.insert("return".into(), TokenKind::Return);
        m.insert("super".into(), TokenKind::Super);
        m.insert("this".into(), TokenKind::This);
        m.insert("true".into(), TokenKind::True);
        m.insert("var".into(), TokenKind::Var);
        m.insert("while".into(), TokenKind::While);
        m
    };
}

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,

    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token>, ScanError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            lexeme: "".into(),
            literal: LoxValue::Nil,
            line: self.line,
            scanner_index: self.start,
        });
        Ok(self.tokens)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<(), ScanError> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenKind::LeftParen),
            ')' => self.add_token(TokenKind::RightParen),
            '{' => self.add_token(TokenKind::LeftBrace),
            '}' => self.add_token(TokenKind::RightBrace),
            ',' => self.add_token(TokenKind::Comma),
            '.' => self.add_token(TokenKind::Dot),
            '-' => self.add_token(TokenKind::Minus),
            '+' => self.add_token(TokenKind::Plus),
            ';' => self.add_token(TokenKind::Semicolon),
            '*' => self.add_token(TokenKind::Star),
            '!' => {
                let kind = if self.match_lookahead('=') {
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                };
                self.add_token(kind)
            }
            '=' => {
                let kind = if self.match_lookahead('=') {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                };
                self.add_token(kind)
            }
            '<' => {
                let kind = if self.match_lookahead('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                };
                self.add_token(kind)
            }
            '>' => {
                let kind = if self.match_lookahead('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                };
                self.add_token(kind)
            }
            '/' => {
                if self.match_lookahead('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenKind::Slash);
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            '"' => self.string()?,
            c if c.is_digit(10) => self.number(),
            c if c == '_' || c.is_alphabetic() => self.identifier(),
            c => return Err(ScanError::UnexpectedCharacter(c, self.line)),
        }
        Ok(())
    }

    fn match_lookahead(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn advance(&mut self) -> char {
        let result = self.source[self.current as usize];
        self.current += 1;
        result
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn add_token(&mut self, kind: TokenKind) {
        let value = match kind {
            TokenKind::True => LoxValue::Bool(true),
            TokenKind::False => LoxValue::Bool(false),
            _ => LoxValue::Nil,
        };
        self.add_literal_token(kind, value);
    }

    fn add_literal_token(&mut self, kind: TokenKind, literal: LoxValue) {
        let text: String = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token {
            kind,
            lexeme: text,
            literal,
            line: self.line,
            scanner_index: self.start,
        });
    }

    fn string(&mut self) -> Result<(), ScanError> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            return Err(ScanError::UnterminatedString(self.line));
        }
        self.advance();
        let value: String = self.source[self.start + 1..self.current - 1]
            .iter()
            .cloned()
            .collect();
        self.add_literal_token(TokenKind::String, LoxValue::Str(value));
        Ok(())
    }

    fn number(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.advance();
            while self.peek().is_digit(10) {
                self.advance();
            }
        }
        let value: f64 = self.source[self.start..self.current]
            .iter()
            .cloned()
            .collect::<String>()
            .parse()
            .expect("Couldn't parse number");
        self.add_literal_token(TokenKind::Number, LoxValue::Float(value));
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() {
            self.advance();
        }
        let text = self.source[self.start..self.current]
            .iter()
            .cloned()
            .collect::<String>();
        let kind = RESERVED_WORDS
            .get(&text)
            .copied()
            .unwrap_or(TokenKind::Identifier);
        self.add_token(kind);
    }
}

#[derive(Debug)]
pub enum ScanError {
    UnexpectedCharacter(char, usize),
    UnterminatedString(usize),
}
impl Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::UnexpectedCharacter(c, line) => write!(f, "[Line {}] Unexpected character '{}'.", line, c),
            ScanError::UnterminatedString(line) => write!(f, "[Line {}] Unterminated string.", line),
        }
    }
}
impl Error for ScanError { }