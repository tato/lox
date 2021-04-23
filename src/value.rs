use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum LoxValue {
    Bool(bool),
    // Int(i64),
    Float(f64),
    Str(String),
    Nil,
}
impl Display for LoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxValue::Bool(x) => write!(f, "{}", x),
            // LiteralValue::Int(x) => write!(f, "{}", x),
            LoxValue::Float(x) => write!(f, "{}", x),
            LoxValue::Str(x) => write!(f, "{}", x),
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
