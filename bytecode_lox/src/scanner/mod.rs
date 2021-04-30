use std::{
    cell::{Cell, RefCell},
    str::CharIndices,
};

mod token;
pub use token::{Token, TokenKind};

use crate::iterator::sneakable::Sneakable;

pub struct Scanner<'source> {
    // characters in the source string are iterated by grapheme clusters.
    // the scanner needs to be able to peek the next two characters as it is
    // currently implemented, but 'core::iter::Peekable' only allows
    // peeking the character right next to the cursor. I implemented a
    // 'Sneakable' type that stores the previous, next and next next elements.
    // it is simple and there is a high chance that it is slow. we will
    // figure it out.
    start: RefCell<Sneakable<CharIndices<'source>>>,
    current: RefCell<Sneakable<CharIndices<'source>>>,
    source: &'source str,
    line: Cell<usize>,
}

impl<'source> Scanner<'source> {
    pub fn new(source: &'source str) -> Scanner<'source> {
        let iter = Sneakable::new(source.char_indices());
        Self {
            start: RefCell::new(iter.clone()),
            current: RefCell::new(iter),
            source,
            line: Cell::new(1),
        }
    }

    pub fn scan(&'source self) -> Token<'source> {
        self.skip_whitespace();

        {
            *self.start.borrow_mut() = self.current.borrow().clone();
        }

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
        self.current.borrow_mut().peek().is_none()
    }

    // panics if the current iterator is at the end
    fn advance(&self) -> char {
        self.current.borrow_mut().next();
        self.current.borrow_mut().previous().unwrap().1
    }

    fn check(&self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != Some(expected) {
            false
        } else {
            self.current.borrow_mut().next();
            true
        }
    }

    fn peek(&self) -> Option<char> {
        self.current.borrow_mut().peek().map(|c| c.1)
    }

    fn peek_next(&self) -> Option<char> {
        self.current.borrow_mut().peek_next().map(|c| c.1)
    }

    fn identifier(&'source self) -> Token<'source> {
        while self
            .peek()
            .map(|c| c.is_alphabetic() || c.is_ascii_digit())
            .unwrap_or(false)
        {
            self.advance();
        }
        self.make_token(self.identifier_type())
    }

    // TODO! measure performance against a simple 'match'
    fn identifier_type(&self) -> TokenKind {
        // interior mutability is weird
        let (start_peek, start_peek_next, current_peek) = {
            let mut start = self.start.borrow_mut();
            let mut current = self.current.borrow_mut();
            let start_peek = start.peek().unwrap().clone();
            let start_peek_next = start.peek_next().unwrap().clone();
            let current_peek = current.peek().unwrap().clone();
            (start_peek, start_peek_next, current_peek)
        };

        let check_keyword = |start: usize, length: usize, rest: &str, kind: TokenKind| {
            // if both slices of source are the same length, and the parts that have not
            // been checked are the same, then it is the token kind you think it is.
            // otherwise it is a simple identifier.
            if current_peek.0 - start_peek.0 == start + length
                && rest == &self.source[start_peek.0 + start..start_peek.0 + start + length]
            {
                kind
            } else {
                TokenKind::Identifier
            }
        };

        match start_peek.1 {
            'a' => check_keyword(1, 2, "nd", TokenKind::And),
            'c' => check_keyword(1, 4, "lass", TokenKind::Class),
            'e' => check_keyword(1, 3, "lse", TokenKind::Else),
            'f' if current_peek.0 - start_peek.0 > 1 => match start_peek_next.1 {
                'a' => check_keyword(2, 3, "lse", TokenKind::False),
                'o' => check_keyword(2, 1, "r", TokenKind::For),
                'u' => check_keyword(2, 1, "n", TokenKind::Fun),
                _ => TokenKind::Identifier,
            },
            'i' => check_keyword(1, 1, "f", TokenKind::If),
            'n' => check_keyword(1, 2, "il", TokenKind::Nil),
            'o' => check_keyword(1, 1, "r", TokenKind::Or),
            'p' => check_keyword(1, 4, "rint", TokenKind::Print),
            'r' => check_keyword(1, 5, "eturn", TokenKind::Return),
            's' => check_keyword(1, 4, "uper", TokenKind::Super),
            't' if current_peek.0 - start_peek.0 > 1 => match start_peek_next.1 {
                'h' => check_keyword(2, 2, "is", TokenKind::This),
                'r' => check_keyword(2, 2, "ue", TokenKind::True),
                _ => TokenKind::Identifier,
            },
            'v' => check_keyword(1, 2, "ar", TokenKind::Var),
            'w' => check_keyword(1, 4, "hile", TokenKind::While),
            _ => TokenKind::Identifier,
        }
    }

    fn number(&'source self) -> Token<'source> {
        while self
            .peek()
            .as_ref()
            .map(char::is_ascii_digit)
            .unwrap_or(false)
        {
            self.advance();
        }
        if self.peek() == Some('.')
            && self
                .peek_next()
                .as_ref()
                .map(char::is_ascii_digit)
                .unwrap_or(false)
        {
            self.advance();
            while self
                .peek()
                .as_ref()
                .map(char::is_ascii_digit)
                .unwrap_or(false)
            {
                self.advance();
            }
        }
        self.make_token(TokenKind::Number)
    }

    fn string(&'source self) -> Token<'source> {
        while self.peek() != Some('"') && !self.is_at_end() {
            if self.peek() == Some('\n') {
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
                Some(' ') | Some('\r') | Some('\t') => {
                    self.advance();
                }
                Some('\n') => {
                    self.line.set(self.line.get() + 1);
                    self.advance();
                }
                Some('/') if self.peek_next() == Some('/') => {
                    while self.peek() != Some('\n') && !self.is_at_end() {
                        self.advance();
                    }
                }
                _ => return,
            }
        }
    }

    fn make_token(&'source self, kind: TokenKind) -> Token<'source> {
        // in the case that kind == Eof, my .peek() calls will return None.
        // in that case, i want the lexeme string to be a 0-length one
        let start_index = self.start.borrow_mut().peek().map(|it| it.0).unwrap_or(0);
        let current_index = self.current.borrow_mut().peek().map(|it| it.0).unwrap_or(0);
        Token {
            kind,
            lexeme: &self.source[start_index..current_index],
            line: self.line.get(),
        }
    }

    fn make_error_token(&'source self, message: &'static str) -> Token<'source> {
        Token {
            kind: TokenKind::Error,
            lexeme: message,
            line: self.line.get(),
        }
    }
}
