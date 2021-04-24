use std::fmt::{Debug, Display};

use crate::interpreter::{Interpreter, InterpreterError};

#[derive(Clone)]
pub struct LoxFunction {
    pub callable: fn(&Interpreter, Vec<LoxValue>) -> Result<LoxValue, InterpreterError>,
    pub args: Vec<String>,
}
impl Debug for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fun {}({})", "Unknown", self.args.join(", "))
    }
}
impl PartialEq for LoxFunction {
    fn eq(&self, _other: &Self) -> bool {
        false // TODO!
    }
}
impl LoxFunction {
    pub fn call(
        &self,
        interpreter: &Interpreter,
        args: Vec<LoxValue>,
    ) -> Result<LoxValue, InterpreterError> {
        (self.callable)(interpreter, args)
    }
    pub fn arity(&self) -> usize {
        self.args.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoxValue {
    Bool(bool),
    Float(f64),
    Str(String),
    Callable(LoxFunction),
    Nil,
}
impl Display for LoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxValue::Bool(x) => write!(f, "{}", x),
            LoxValue::Float(x) => write!(f, "{}", x),
            LoxValue::Str(x) => write!(f, "{}", x),
            LoxValue::Callable(x) => write!(f, "{:?}", x),
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
}

// pub struct LoxCallable {
// }
// impl LoxCallable {
//     pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<LoxValue>) -> LoxValue {
//     }
// }
