use std::{
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
};

use crate::{
    ast::Stmt,
    environment::Environment,
    interpreter::{Interpreter, InterpreterError},
    token::Token,
};

pub trait LoxCallable {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<LoxValue>,
    ) -> Result<LoxValue, InterpreterError>;
    fn arity(&self) -> usize;
}

#[derive(Clone)]
pub struct BuiltInFunction {
    name: String,
    args: Vec<String>,
    callable: fn(&Interpreter, Vec<LoxValue>) -> Result<LoxValue, InterpreterError>,
}
impl Debug for BuiltInFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fun {}({})>", self.name, self.args.join(", "))
    }
}
impl PartialEq for BuiltInFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl BuiltInFunction {
    pub fn new(
        name: &str,
        args: Vec<&str>,
        callable: fn(&Interpreter, Vec<LoxValue>) -> Result<LoxValue, InterpreterError>,
    ) -> Self {
        Self {
            name: name.into(),
            args: args.into_iter().map(str::to_string).collect(),
            callable,
        }
    }
}
impl LoxCallable for BuiltInFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<LoxValue>,
    ) -> Result<LoxValue, InterpreterError> {
        (self.callable)(interpreter, args)
    }
    fn arity(&self) -> usize {
        self.args.len()
    }
}

#[derive(Clone)]
pub struct UserFunction {
    name: Box<Token>,
    args: Vec<Token>,
    body: Vec<Stmt>,
    closure: Arc<Mutex<Environment>>,
}
impl Debug for UserFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<fun {}({})>",
            self.name.lexeme,
            self.args
                .iter()
                .map(|it| &it.lexeme)
                .map(|it| &**it)
                .collect::<Vec<&str>>()
                .join(", ")
        )
    }
}
impl PartialEq for UserFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name.lexeme == other.name.lexeme && self.name.line == other.name.line
        // TODO! !?
    }
}
impl UserFunction {
    pub fn new(
        name: &Token,
        args: &[Token],
        body: &[Stmt],
        closure: Arc<Mutex<Environment>>,
    ) -> Self {
        Self {
            name: name.clone().into(),
            args: args.to_vec(),
            body: body.to_vec(),
            closure,
        }
    }
}
impl LoxCallable for UserFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<LoxValue>,
    ) -> Result<LoxValue, InterpreterError> {
        let environment = Environment::new_child(self.closure.clone());
        for (arg, arg_value) in self.args.iter().zip(&args) {
            environment
                .lock()
                .unwrap()
                .define(&arg.lexeme, arg_value.clone());
        }
        if let Err(e) = interpreter.execute_block(&self.body, environment) {
            match e {
                InterpreterError::Return(v) => Ok(v),
                e => Err(e),
            }
        } else {
            Ok(LoxValue::Nil)
        }
    }
    fn arity(&self) -> usize {
        self.args.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoxValue {
    Bool(bool),
    Float(f64),
    Str(String),
    BuiltInFunction(Box<BuiltInFunction>),
    UserFunction(Box<UserFunction>),
    Nil,
}
impl Display for LoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxValue::Bool(x) => write!(f, "{}", x),
            LoxValue::Float(x) => write!(f, "{}", x),
            LoxValue::Str(x) => write!(f, "{}", x),
            LoxValue::BuiltInFunction(x) => write!(f, "{:?}", x),
            LoxValue::UserFunction(x) => write!(f, "{:?}", x),
            LoxValue::Nil => write!(f, "nil"),
        }
    }
}

impl LoxValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            LoxValue::Bool(v) => *v,
            LoxValue::Nil => false,
            _ => true,
        }
    }
    pub fn equals(&self, other: &LoxValue) -> bool {
        self == other
    }
    pub fn as_callable(&self) -> Option<&dyn LoxCallable> {
        match self {
            LoxValue::BuiltInFunction(x) => Some(x.as_ref()),
            LoxValue::UserFunction(x) => Some(x.as_ref()),
            _ => None,
        }
    }
}

// pub struct LoxCallable {
// }
// impl LoxCallable {
//     pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<LoxValue>) -> LoxValue {
//     }
// }
