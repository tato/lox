use crate::{
    ast::{Expr, Stmt},
    environment::Environment,
    token::{Token, TokenKind},
    value::{LoxFunction, LoxValue},
};
use std::{
    cell::RefCell,
    error::Error,
    fmt::Display,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Interpreter {
    _globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
}
impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new();
        globals.borrow_mut().define(
            "clock".into(),
            LoxValue::Callable(LoxFunction {
                args: vec![],
                callable: |_: &Interpreter,
                           _: Vec<LoxValue>|
                 -> Result<LoxValue, InterpreterError> {
                    Ok(LoxValue::Float(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map_err(|_| InterpreterError::Internal)?
                            .as_millis() as f64,
                    ))
                },
            }),
        );

        Self {
            _globals: globals.clone(),
            environment: globals.clone(),
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) {
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
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Variable { name } => self
                .environment
                .borrow()
                .get(&name.lexeme())
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

                if let LoxValue::Callable(fun) = &callee {
                    if arguments.len() != fun.arity() {
                        Err(InterpreterError::FunctionArity(
                            paren.clone(),
                            fun.arity(),
                            arguments.len(),
                        ))
                    } else {
                        fun.call(self, arguments)
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
                    .borrow_mut()
                    .assign(name.lexeme(), value.clone())
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
                } else {
                    if !left.is_truthy() {
                        return Ok(left);
                    }
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
            Stmt::Var { name, initializer } => {
                let value = if let Some(expr) = initializer {
                    self.evaluate(expr)?
                } else {
                    LoxValue::Nil
                };
                self.environment.borrow_mut().define(name.lexeme(), value);
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
        };
        Ok(())
    }

    fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
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
                write!(f, "Undefined variable '{}'.", tok.lexeme())
            }
            InterpreterError::NotCallable(val) => {
                write!(f, "'{}' is not callable.", val)
            }
            InterpreterError::FunctionArity(_at, expected, got) => {
                write!(f, "Expected {} arguments but got {}.", expected, got)
            }
        }
    }
}
impl Error for InterpreterError {}
