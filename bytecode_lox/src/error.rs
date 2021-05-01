use std::fmt::Display;

use crate::{
    scanner::{Token, TokenKind},
    value::Value,
};

#[derive(thiserror::Error, Debug)]
pub enum InterpretError {
    #[error(transparent)]
    Compile(#[from] CompileError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}

#[derive(thiserror::Error, Debug)]
pub enum CompileError {
    #[error("{0}")]
    ScanError(ErrorInfo),
    #[error("{0}")]
    ParseError(ErrorInfo),
}

#[derive(thiserror::Error, Debug)]
pub enum RuntimeError {
    #[error("Byte '{0}' does not map to any op code.")]
    InvalidOpcode(u8),
    #[error("Operand for {0} must be number, but was {1}.")]
    OperandMustBeNumber(String, Value),
}

#[derive(Debug)]
pub struct ErrorInfo {
    line: usize,
    location: String,
    message: String,
}
impl Display for ErrorInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] {}", self.line, self.message)
    }
}
impl ErrorInfo {
    pub fn error<'any>(token: &Token<'any>, message: &str) -> Self {
        let (location, message) = if token.kind == TokenKind::Eof {
            (" at end".to_string(), message.to_string())
        } else if token.kind == TokenKind::Error {
            ("".to_string(), token.lexeme.to_string())
        } else {
            (
                format!(" at '{}'", token.lexeme.to_string()),
                message.to_string(),
            )
        };
        Self {
            line: token.line,
            location,
            message,
        }
    }
}
