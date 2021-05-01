use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
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
}