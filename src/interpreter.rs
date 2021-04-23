use std::{error::Error, fmt::Display};

use crate::{ast::{Expr, Visitor}, error::ErrorReporter, token::TokenKind, value::LoxValue};

pub struct Interpreter {}
impl Interpreter {
    pub fn interpret(&mut self, expression: &mut Expr, reporter: &mut ErrorReporter) {
        let result = expression.accept(self);
        match result {
            Ok(val) => println!("{}", val.to_string()),
            Err(e) => eprintln!("{}", e),
        }
    }
}
impl Visitor for Interpreter {
    type Return = Result<LoxValue, InterpreterError>;

    fn visit(&mut self, expr: &mut Expr) -> Self::Return {
        match expr {
            Expr::Literal{ value } => Ok(value.clone()),
            Expr::Grouping{ expression } => expression.accept(self),
            Expr::Unary{ operator, right } => {
                let right = right.accept(self)?;
                match operator.kind {
                    TokenKind::Minus => match right {
                        LoxValue::Float(f) => Ok(LoxValue::Float(-f)),
                        v => Err(InterpreterError::UnaryMinusOperandMustBeNumber(v))
                    },
                    TokenKind::Bang => Ok(LoxValue::Bool(!right.is_truthy())),
                    _ => Err(InterpreterError::Internal)
                }
            },
            Expr::Binary{ left, operator, right } => {
                let left = left.accept(self)?;
                let right = right.accept(self)?;

                match operator.kind {
                    TokenKind::Minus => {
                        if let (LoxValue::Float(l), LoxValue::Float(r)) = (&left, &right) {
                            Ok(LoxValue::Float(l - r))
                        } else {
                            Err(InterpreterError::OperandsMustBeNumbers)
                        }
                    }
                    TokenKind::Slash => {
                        if let (LoxValue::Float(l), LoxValue::Float(r)) = (&left, &right) {
                            Ok(LoxValue::Float(l / r))
                        } else {
                            Err(InterpreterError::OperandsMustBeNumbers)
                        }
                    }
                    TokenKind::Star => {
                        if let (LoxValue::Float(l), LoxValue::Float(r)) = (&left, &right) {
                            Ok(LoxValue::Float(l * r))
                        } else {
                            Err(InterpreterError::OperandsMustBeNumbers)
                        }
                    }
                    TokenKind::Plus => {
                        if let (LoxValue::Float(l), LoxValue::Float(r)) = (&left, &right) {
                            Ok(LoxValue::Float(l + r))
                        } else if let (LoxValue::Str(l), LoxValue::Str(r)) = (&left, &right) {
                            Ok(LoxValue::Str(l.clone() + r))
                        } else {
                            Err(InterpreterError::OperandsMustBeNumbersOrStr)
                        }
                    }
                    TokenKind::Greater => {
                        if let (LoxValue::Float(l), LoxValue::Float(r)) = (&left, &right) {
                            Ok(LoxValue::Bool(l > r))
                        } else {
                            Err(InterpreterError::OperandsMustBeNumbers)
                        }
                    }
                    TokenKind::GreaterEqual => {
                        if let (LoxValue::Float(l), LoxValue::Float(r)) = (&left, &right) {
                            Ok(LoxValue::Bool(l >= r))
                        } else {
                            Err(InterpreterError::OperandsMustBeNumbers)
                        }
                    }
                    TokenKind::Less => {
                        if let (LoxValue::Float(l), LoxValue::Float(r)) = (&left, &right) {
                            Ok(LoxValue::Bool(l < r))
                        } else {
                            Err(InterpreterError::OperandsMustBeNumbers)
                        }
                    }
                    TokenKind::LessEqual => {
                        if let (LoxValue::Float(l), LoxValue::Float(r)) = (&left, &right) {
                            Ok(LoxValue::Bool(l <= r))
                        } else {
                            Err(InterpreterError::OperandsMustBeNumbers)
                        }
                    }
                    TokenKind::BangEqual => {
                        Ok(LoxValue::Bool(!left.equals(&right)))
                    }
                    TokenKind::EqualEqual => {
                        Ok(LoxValue::Bool(left.equals(&right)))
                    }
                    _ => Err(InterpreterError::Internal),
                }
            },
        }
    }
}

#[derive(Debug)]
pub enum InterpreterError {
    Internal,
    UnaryMinusOperandMustBeNumber(LoxValue),
    OperandsMustBeNumbers,
    OperandsMustBeNumbersOrStr,
}
impl Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::Internal =>
                writeln!(f, "Unexpected error."),
            InterpreterError::UnaryMinusOperandMustBeNumber(v) =>
                write!(f, "Unary minus must be applied to number, but value was {}.", v),
            InterpreterError::OperandsMustBeNumbers =>
                write!(f, "Operands must be numbers."),
            InterpreterError::OperandsMustBeNumbersOrStr =>
                write!(f, "Operands must be numbers or strings."),
        }
    }
}
impl Error for InterpreterError { }