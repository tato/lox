use std::{error::Error, fmt::Display};

use crate::{ast::{Expr, Stmt}, environment::Environment, token::{Token, TokenKind}, value::LoxValue};

pub struct Interpreter {
    environment: Environment,
}
impl Interpreter {
    pub fn new() -> Self {
        Self { environment: Environment::new() }
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) {
        for statement in statements {
            match self.execute(statement) {
                Ok(_) => { },
                Err(e) => {
                    eprintln!("{}", e);
                    break;
                },
            }
        }
    }

    fn evaluate(&self, expr: &Expr) -> Result<LoxValue, InterpreterError> {
        match expr {
            Expr::Literal{ value } => Ok(value.clone()),
            Expr::Variable{ name } => {
                self.environment.get(&name).ok_or_else(|| {
                    InterpreterError::UndefinedVariable(name.clone())
                })
            },
            Expr::Grouping{ expression } => self.evaluate(expression),
            Expr::Unary{ operator, right } => {
                let right = self.evaluate(right)?;
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
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

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

    fn execute(&mut self, stmt: &Stmt) -> Result<(), InterpreterError> {
        match stmt {
            Stmt::Expression{ expression } => {
                self.evaluate(expression)?;
            },
            Stmt::Print{ expression } => {
                let value = self.evaluate(expression)?;
                println!("{}", value.to_string());
            },
            Stmt::Var{ name, initializer } => {
                let value = if let Some(expr) = initializer {
                    self.evaluate(expr)?
                } else {
                    LoxValue::Nil   
                };
                self.environment.define(name.lexeme(), value);
            }
        };
        Ok(())
    }
}

#[derive(Debug)]
pub enum InterpreterError {
    Internal,
    UnaryMinusOperandMustBeNumber(LoxValue),
    OperandsMustBeNumbers,
    OperandsMustBeNumbersOrStr,
    UndefinedVariable(Token),
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
            InterpreterError::UndefinedVariable(tok) => 
                write!(f, "Undefined variable '{}'.", tok.lexeme())
        }
    }
}
impl Error for InterpreterError { }