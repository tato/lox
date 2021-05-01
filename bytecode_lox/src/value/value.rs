use super::Obj;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    Obj(Obj),
}

impl Default for Value {
    fn default() -> Self {
        Value::Nil
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Value::*;
        match self {
            Bool(x) => write!(f, "{}", x),
            Nil => write!(f, "nil"),
            Number(x) => write!(f, "{}", x),
            Obj(x) => write!(f, "{}", x),
        }
    }
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        use Value::*;
        match self {
            Bool(x) => !x,
            Nil => true,
            _ => false,
        }
    }

    pub fn equals(&self, other: &Value) -> bool {
        self == other
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::Obj(x) => x.as_string(),
            _ => None,
        }
    }
}
