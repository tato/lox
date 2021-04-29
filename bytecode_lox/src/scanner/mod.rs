use std::cell::Cell;

mod token;
pub use token::{Token, TokenKind};

pub struct Scanner<'source> {
    start: Cell<usize>,
    current: Cell<usize>,
    source: &'source [char],
    line: Cell<usize>,
}

impl<'source> Scanner<'source> {
    pub fn new(source: &'source [char]) -> Scanner<'source> {
        Self {
            start: Cell::new(0),
            current: Cell::new(0),
            source,
            line: Cell::new(1),
        }
    }

    pub fn scan(&'source self) -> Token<'source> {
        self.skip_whitespace();

        self.start.set(self.current.get());

        if self.is_at_end() {
            return self.make_token(TokenKind::Eof);
        }

        let c = self.advance();

        match c {
            '(' => self.make_token(TokenKind::LeftParen),
            ')' => self.make_token(TokenKind::RightParen),
            '{' => self.make_token(TokenKind::LeftBrace),
            '}' => self.make_token(TokenKind::RightBrace),
            ';' => self.make_token(TokenKind::Semicolon),
            ',' => self.make_token(TokenKind::Comma),
            '.' => self.make_token(TokenKind::Dot),
            '-' => self.make_token(TokenKind::Minus),
            '+' => self.make_token(TokenKind::Plus),
            '/' => self.make_token(TokenKind::Slash),
            '*' => self.make_token(TokenKind::Star),
            '!' => self.make_token(if self.check('=') {
                TokenKind::BangEqual
            } else {
                TokenKind::Bang
            }),
            '=' => self.make_token(if self.check('=') {
                TokenKind::EqualEqual
            } else {
                TokenKind::Equal
            }),
            '<' => self.make_token(if self.check('=') {
                TokenKind::LessEqual
            } else {
                TokenKind::Less
            }),
            '>' => self.make_token(if self.check('=') {
                TokenKind::GreaterEqual
            } else {
                TokenKind::Greater
            }),
            '"' => self.string(),
            _ if c.is_alphabetic() => self.identifier(),
            _ if c.is_ascii_digit() => self.number(),
            _ => self.make_error_token("Unexpected character."),
        }
    }

    fn is_at_end(&self) -> bool {
        self.source[self.current.get()] == '\0'
    }

    fn advance(&self) -> char {
        self.current.set(self.current.get() + 1);
        self.source[self.current.get() - 1]
    }

    fn check(&self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        } else if self.source[self.current.get()] != expected {
            false
        } else {
            self.current.set(self.current.get() + 1);
            true
        }
    }

    fn peek(&self) -> char {
        self.source[self.current.get()]
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current.get() + 1]
        }
    }

    fn identifier(&self) -> Token {
        while self.peek().is_alphabetic() || self.peek().is_ascii_digit() {
            self.advance();
        }
        self.make_token(self.identifier_type())
    }

    fn identifier_type(&self) -> TokenKind {
        // TODO! measure performance against a simple 'match'
        match self.source[self.start.get()] {
            'a' => self.check_keyword(1, 2, "nd", TokenKind::And),
            'c' => self.check_keyword(1, 4, "lass", TokenKind::Class),
            'e' => self.check_keyword(1, 3, "lse", TokenKind::Else),
            'f' if self.current.get() - self.start.get() > 1 => {
                match self.source[self.start.get() + 1] {
                    'a' => self.check_keyword(2, 3, "lse", TokenKind::False),
                    'o' => self.check_keyword(2, 1, "r", TokenKind::For),
                    'u' => self.check_keyword(2, 1, "n", TokenKind::Fun),
                    _ => TokenKind::Identifier,
                }
            }
            'i' => self.check_keyword(1, 1, "f", TokenKind::If),
            'n' => self.check_keyword(1, 2, "il", TokenKind::Nil),
            'o' => self.check_keyword(1, 1, "r", TokenKind::Or),
            'p' => self.check_keyword(1, 4, "rint", TokenKind::Print),
            'r' => self.check_keyword(1, 5, "eturn", TokenKind::Return),
            's' => self.check_keyword(1, 4, "uper", TokenKind::Super),
            't' if self.current.get() - self.start.get() > 1 => {
                match self.source[self.start.get() + 1] {
                    'h' => self.check_keyword(2, 2, "is", TokenKind::This),
                    'r' => self.check_keyword(2, 2, "ue", TokenKind::True),
                    _ => TokenKind::Identifier,
                }
            }
            'v' => self.check_keyword(1, 2, "ar", TokenKind::Var),
            'w' => self.check_keyword(1, 4, "hile", TokenKind::While),
            _ => TokenKind::Identifier,
        }
    }

    fn check_keyword(&self, start: usize, length: usize, rest: &str, kind: TokenKind) -> TokenKind {
        if self.current.get() - self.start.get() == start + length
            && rest.chars().collect::<Vec<char>>().as_slice()
                == &self.source[(self.start.get() + start)..(self.start.get() + start + length)]
        {
            kind
        } else {
            TokenKind::Identifier
        }
    }

    fn number(&self) -> Token {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        self.make_token(TokenKind::Number)
    }

    fn string(&self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line.set(self.line.get() + 1);
            }
            self.advance();
        }

        if self.is_at_end() {
            self.make_error_token("Unterminated string.")
        } else {
            self.advance();
            self.make_token(TokenKind::String)
        }
    }

    fn skip_whitespace(&self) {
        loop {
            let c = self.peek();
            match c {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line.set(self.line.get() + 1);
                    self.advance();
                }
                '/' if self.peek_next() == '/' => {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                }
                _ => return,
            }
        }
    }

    fn make_token(&'source self, kind: TokenKind) -> Token<'source> {
        Token {
            kind,
            lexeme: &self.source[self.start.get()..self.current.get()],
            line: self.line.get(),
        }
    }

    fn make_error_token(&'source self, _message: &str) -> Token<'source> {
        Token {
            kind: TokenKind::Error,
            lexeme: &['t', 'o', 'd', 'o'],
            line: self.line.get(),
        }
    }
}
