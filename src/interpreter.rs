use crate::{
    ast::{Expr, Stmt},
    environment::Environment,
    token::{Token, TokenKind},
    value::{BuiltInFunction, LoxValue, UserFunction},
};
use std::{collections::HashMap, error::Error, fmt::Display, sync::{Arc, Mutex}, time::{SystemTime, UNIX_EPOCH}};

pub struct Interpreter {
    _globals: Arc<Mutex<Environment>>,
    environment: Arc<Mutex<Environment>>,
    locals: HashMap<Expr, usize>,
}
impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new();
        globals.lock().unwrap().define(
            "clock".into(),
            LoxValue::BuiltInFunction(BuiltInFunction::new("clock", vec![], |_, _| {
                Ok(LoxValue::Float(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|_| InterpreterError::Internal)?
                        .as_millis() as f64,
                ))
            }).into()),
        );

        Self {
            _globals: globals.clone(),
            environment: globals,
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) {
        for statement in statements {
            match self.execute(statement) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    break;
                }
            }
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<LoxValue, InterpreterError> {
        match expr {
            Expr::Literal { value } => Ok(value.literal.clone()),
            Expr::Variable { name } => self
                .environment
                .lock()
                .unwrap()
                .get(&name.lexeme)
                .ok_or_else(|| InterpreterError::UndefinedVariable(name.clone())),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.evaluate(callee)?;
                let arguments = arguments
                    .iter()
                    .map(|it| self.evaluate(it))
                    .collect::<Result<Vec<LoxValue>, InterpreterError>>()?;

                if let Some(callable) = callee.as_callable() {
                    if arguments.len() != callable.arity() {
                        Err(InterpreterError::FunctionArity(
                            paren.clone(),
                            callable.arity(),
                            arguments.len(),
                        ))
                    } else {
                        callable.call(self, arguments)
                    }
                } else {
                    Err(InterpreterError::NotCallable(callee))
                }
            }
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;
                match operator.kind {
                    TokenKind::Minus => match right {
                        LoxValue::Float(f) => Ok(LoxValue::Float(-f)),
                        v => Err(InterpreterError::UnaryMinusOperandMustBeNumber(v)),
                    },
                    TokenKind::Bang => Ok(LoxValue::Bool(!right.is_truthy())),
                    _ => Err(InterpreterError::Internal),
                }
            }
            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;
                self.environment
                    .lock()
                    .unwrap()
                    .assign(&name.lexeme, value.clone())
                    .ok_or_else(|| InterpreterError::UndefinedVariable(name.clone()))?;
                Ok(value)
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
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
                    TokenKind::BangEqual => Ok(LoxValue::Bool(!left.equals(&right))),
                    TokenKind::EqualEqual => Ok(LoxValue::Bool(left.equals(&right))),
                    _ => Err(InterpreterError::Internal),
                }
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;

                if operator.kind == TokenKind::Or {
                    if left.is_truthy() {
                        return Ok(left);
                    }
                } else if !left.is_truthy() {
                    return Ok(left);
                } 
                self.evaluate(right)
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), InterpreterError> {
        match stmt {
            Stmt::Expression { expression } => {
                self.evaluate(expression)?;
            }
            Stmt::Print { expression } => {
                let value = self.evaluate(expression)?;
                println!("{}", value.to_string());
            }
            Stmt::Return { value, .. } => {
                let value = if let Some(v) = value {
                    self.evaluate(v)?
                } else {
                    LoxValue::Nil
                };
                return Err(InterpreterError::Return(value));
            }
            Stmt::Var { name, initializer } => {
                let value = if let Some(expr) = initializer {
                    self.evaluate(expr)?
                } else {
                    LoxValue::Nil
                };
                self.environment.lock().unwrap().define(&name.lexeme, value);
            }
            Stmt::Block { statements } => {
                self.execute_block(statements, Environment::new_child(self.environment.clone()))?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.evaluate(condition)?.is_truthy() {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }
            }
            Stmt::While { condition, body } => {
                while self.evaluate(condition)?.is_truthy() {
                    self.execute(body)?;
                }
            }
            Stmt::Function { name, params, body } => {
                let function = UserFunction::new(name, params, body, self.environment.clone());
                self.environment
                    .lock()
                    .unwrap()
                    .define(&name.lexeme, LoxValue::UserFunction(function.into()));
            }
        };
        Ok(())
    }

    pub fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Arc<Mutex<Environment>>,
    ) -> Result<(), InterpreterError> {
        let previous = self.environment.clone();
        self.environment = environment;
        for statement in statements {
            let result = self.execute(statement);
            if result.is_err() {
                self.environment = previous;
                return result;
            }
        }

        self.environment = previous;
        Ok(())
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        self.locals.insert(expr.clone(), depth);
    }
}

#[derive(Debug)]
pub enum InterpreterError {
    Internal,
    UnaryMinusOperandMustBeNumber(LoxValue),
    OperandsMustBeNumbers,
    OperandsMustBeNumbersOrStr,
    UndefinedVariable(Token),
    NotCallable(LoxValue),
    FunctionArity(Token, usize, usize),
    Return(LoxValue),
}
impl Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::Internal => writeln!(f, "Unexpected error."),
            InterpreterError::UnaryMinusOperandMustBeNumber(v) => write!(
                f,
                "Unary minus must be applied to number, but value was {}.",
                v
            ),
            InterpreterError::OperandsMustBeNumbers => write!(f, "Operands must be numbers."),
            InterpreterError::OperandsMustBeNumbersOrStr => {
                write!(f, "Operands must be numbers or strings.")
            }
            InterpreterError::UndefinedVariable(tok) => {
                write!(f, "Undefined variable '{}'.", tok.lexeme)
            }
            InterpreterError::NotCallable(val) => {
                write!(f, "'{}' is not callable.", val)
            }
            InterpreterError::FunctionArity(_at, expected, got) => {
                write!(f, "Expected {} arguments but got {}.", expected, got)
            }
            InterpreterError::Return(_) => write!(f, "INTERNAL ERROR: Return was not caught."),
        }
    }
}
impl Error for InterpreterError {}
