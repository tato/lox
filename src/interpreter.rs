use crate::{
    ast::{Expr, Stmt},
    environment::Environment,
    token::{Token, TokenKind},
    value::{BuiltInFunction, ClassDefinition, RuntimeValue, UserFunction},
};
use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Interpreter {
    globals: Arc<Environment>,
    environment: Arc<Environment>,
    locals: HashMap<Expr, usize>,
}
impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new();
        globals.define(
            "clock".into(),
            RuntimeValue::BuiltInFunction(
                BuiltInFunction::new("clock", vec![], |_, _| {
                    Ok(RuntimeValue::Float(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map_err(|_| InterpreterError::Internal)?
                            .as_millis() as f64,
                    ))
                })
                .into(),
            ),
        );

        Self {
            globals: globals.clone(),
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

    fn evaluate(&mut self, expr: &Expr) -> Result<RuntimeValue, InterpreterError> {
        match expr {
            Expr::Literal { value } => Ok(value.literal.clone()),
            Expr::Variable { name } => self.look_up_variable(name, expr),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.evaluate(callee)?;
                let arguments = arguments
                    .iter()
                    .map(|it| self.evaluate(it))
                    .collect::<Result<Vec<RuntimeValue>, InterpreterError>>()?;

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
            Expr::Get { object, name } => {
                let object = self.evaluate(object)?;
                if let RuntimeValue::Instance(instance) = object {
                    instance
                        .get(name)
                        .ok_or_else(|| InterpreterError::UndefinedProperty(name.clone()))
                } else {
                    Err(InterpreterError::MustAccessValueOnInstances)
                }
            }
            Expr::Set {
                name,
                object,
                value,
            } => {
                let object = self.evaluate(object)?;
                if let RuntimeValue::Instance(instance) = object {
                    let value = self.evaluate(value)?;
                    instance.set(name, value.clone());
                    Ok(value)
                } else {
                    Err(InterpreterError::MustAccessValueOnInstances)
                }
            }
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;
                match operator.kind {
                    TokenKind::Minus => match right {
                        RuntimeValue::Float(f) => Ok(RuntimeValue::Float(-f)),
                        v => Err(InterpreterError::UnaryMinusOperandMustBeNumber(v)),
                    },
                    TokenKind::Bang => Ok(RuntimeValue::Bool(!right.is_truthy())),
                    _ => Err(InterpreterError::Internal),
                }
            }
            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;
                let distance = self.locals.get(expr);
                if let Some(distance) = distance {
                    self.environment
                        .assign_at(*distance, &name.lexeme, value.clone());
                } else {
                    self.globals.assign(&name.lexeme, value.clone());
                }
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
                        if let (RuntimeValue::Float(l), RuntimeValue::Float(r)) = (&left, &right) {
                            Ok(RuntimeValue::Float(l - r))
                        } else {
                            Err(InterpreterError::OperandsMustBeNumbers)
                        }
                    }
                    TokenKind::Slash => {
                        if let (RuntimeValue::Float(l), RuntimeValue::Float(r)) = (&left, &right) {
                            Ok(RuntimeValue::Float(l / r))
                        } else {
                            Err(InterpreterError::OperandsMustBeNumbers)
                        }
                    }
                    TokenKind::Star => {
                        if let (RuntimeValue::Float(l), RuntimeValue::Float(r)) = (&left, &right) {
                            Ok(RuntimeValue::Float(l * r))
                        } else {
                            Err(InterpreterError::OperandsMustBeNumbers)
                        }
                    }
                    TokenKind::Plus => {
                        if let (RuntimeValue::Float(l), RuntimeValue::Float(r)) = (&left, &right) {
                            Ok(RuntimeValue::Float(l + r))
                        } else if let (RuntimeValue::Str(l), RuntimeValue::Str(r)) = (&left, &right)
                        {
                            let s = l.to_string() + r;
                            Ok(RuntimeValue::Str(s.as_str().into()))
                        } else {
                            Err(InterpreterError::OperandsMustBeNumbersOrStr)
                        }
                    }
                    TokenKind::Greater => {
                        if let (RuntimeValue::Float(l), RuntimeValue::Float(r)) = (&left, &right) {
                            Ok(RuntimeValue::Bool(l > r))
                        } else {
                            Err(InterpreterError::OperandsMustBeNumbers)
                        }
                    }
                    TokenKind::GreaterEqual => {
                        if let (RuntimeValue::Float(l), RuntimeValue::Float(r)) = (&left, &right) {
                            Ok(RuntimeValue::Bool(l >= r))
                        } else {
                            Err(InterpreterError::OperandsMustBeNumbers)
                        }
                    }
                    TokenKind::Less => {
                        if let (RuntimeValue::Float(l), RuntimeValue::Float(r)) = (&left, &right) {
                            Ok(RuntimeValue::Bool(l < r))
                        } else {
                            Err(InterpreterError::OperandsMustBeNumbers)
                        }
                    }
                    TokenKind::LessEqual => {
                        if let (RuntimeValue::Float(l), RuntimeValue::Float(r)) = (&left, &right) {
                            Ok(RuntimeValue::Bool(l <= r))
                        } else {
                            Err(InterpreterError::OperandsMustBeNumbers)
                        }
                    }
                    TokenKind::BangEqual => Ok(RuntimeValue::Bool(!left.equals(&right))),
                    TokenKind::EqualEqual => Ok(RuntimeValue::Bool(left.equals(&right))),
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
                    RuntimeValue::Nil
                };
                return Err(InterpreterError::Return(value));
            }
            Stmt::Var { name, initializer } => {
                let value = if let Some(expr) = initializer {
                    self.evaluate(expr)?
                } else {
                    RuntimeValue::Nil
                };
                self.environment.define(&name.lexeme, value);
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
            Stmt::Function(fun) => {
                let function = UserFunction::new(fun, self.environment.clone());
                self.environment
                    .define(&fun.name.lexeme, RuntimeValue::UserFunction(function.into()));
            }
            Stmt::Class { name, methods } => {
                self.environment.define(&name.lexeme, RuntimeValue::Nil);

                let mut class_methods = HashMap::new();
                for method in methods {
                    let function = UserFunction::new(method, self.environment.clone());
                    class_methods.insert(method.name.lexeme.clone(), function.into());
                }

                let class = RuntimeValue::Class(ClassDefinition::new(name, class_methods).into());
                self.environment.assign(&name.lexeme, class);
            }
        };
        Ok(())
    }

    pub fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Arc<Environment>,
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

    fn look_up_variable(
        &mut self,
        name: &Token,
        expr: &Expr,
    ) -> Result<RuntimeValue, InterpreterError> {
        let distance = self.locals.get(expr);
        let look_up = if let Some(distance) = distance {
            self.environment.get_at(*distance, &name.lexeme)
        } else {
            self.globals.get(&name.lexeme)
        };
        look_up.ok_or_else(|| InterpreterError::UndefinedVariable(name.clone()))
    }
}

#[derive(Debug)]
pub enum InterpreterError {
    Internal,
    UnaryMinusOperandMustBeNumber(RuntimeValue),
    OperandsMustBeNumbers,
    OperandsMustBeNumbersOrStr,
    UndefinedVariable(Token),
    UndefinedProperty(Token),
    NotCallable(RuntimeValue),
    FunctionArity(Token, usize, usize),
    MustAccessValueOnInstances,
    Return(RuntimeValue),
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
            InterpreterError::UndefinedProperty(tok) => {
                write!(f, "Undefined property '{}'.", tok.lexeme)
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
            InterpreterError::MustAccessValueOnInstances => {
                write!(f, "Only instances have properties.")
            }
            InterpreterError::Return(_) => write!(f, "INTERNAL ERROR: Return was not caught."),
        }
    }
}
impl Error for InterpreterError {}
