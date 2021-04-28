use super::RuntimeValue;
use crate::interpreter::{Interpreter, InterpreterError};

pub trait CallableValue {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue, InterpreterError>;
    fn arity(&self) -> usize;
}
