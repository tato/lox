use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

mod callable;
mod class;
mod function;
pub use callable::CallableValue;
pub use class::{ClassDefinition, ClassInstance};
pub use function::{BuiltInFunction, UserFunction};

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Bool(bool),
    Float(f64),
    Str(Arc<str>),
    BuiltInFunction(BuiltInFunction),
    UserFunction(UserFunction),
    Class(ClassDefinition),
    Instance(ClassInstance),
    Nil,
}
impl Display for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeValue::Bool(x) => write!(f, "{}", x),
            RuntimeValue::Float(x) => write!(f, "{}", x),
            RuntimeValue::Str(x) => write!(f, "{}", x),
            RuntimeValue::BuiltInFunction(x) => write!(f, "{}", x),
            RuntimeValue::UserFunction(x) => write!(f, "{}", x),
            RuntimeValue::Class(x) => write!(f, "{}", x),
            RuntimeValue::Instance(x) => write!(f, "{}", x),
            RuntimeValue::Nil => write!(f, "nil"),
        }
    }
}

impl RuntimeValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            RuntimeValue::Bool(v) => *v,
            RuntimeValue::Nil => false,
            _ => true,
        }
    }
    pub fn equals(&self, other: &RuntimeValue) -> bool {
        self == other
    }
    pub fn as_callable(&self) -> Option<&dyn CallableValue> {
        match self {
            RuntimeValue::BuiltInFunction(x) => Some(x),
            RuntimeValue::UserFunction(x) => Some(x),
            RuntimeValue::Class(x) => Some(x),
            _ => None,
        }
    }
}
