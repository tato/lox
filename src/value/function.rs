use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use crate::{
    ast::FunctionStmt,
    environment::Environment,
    interpreter::{Interpreter, InterpreterError},
};

use super::{CallableValue, ClassInstance, RuntimeValue};

struct UserFunctionStorage {
    declaration: FunctionStmt,
    closure: Environment,
}
#[derive(Clone)]
pub struct UserFunction(Arc<UserFunctionStorage>);

impl Debug for UserFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UserFunction{{ name: {:?}, args: {:?}, body: {:?}, closure: ?? }}",
            self.0.declaration.name, self.0.declaration.params, self.0.declaration.body
        )
    }
}
impl Display for UserFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<fun {}({})>",
            self.0.declaration.name.lexeme,
            self.0
                .declaration
                .params
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
        self.0.declaration.name == other.0.declaration.name
    }
}
impl UserFunction {
    pub fn new(fun: &FunctionStmt, closure: &Environment) -> Self {
        Self(
            UserFunctionStorage {
                declaration: fun.clone(),
                closure: closure.clone(),
            }
            .into(),
        )
    }
    pub fn bind(&self, instance: &ClassInstance) -> UserFunction {
        let environment = self.0.closure.child();
        environment.define("this", RuntimeValue::Instance(instance.clone()));
        UserFunction::new(&self.0.declaration, &environment)
    }
}
impl CallableValue for UserFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue, InterpreterError> {
        let environment = self.0.closure.child();
        for (arg, arg_value) in self.0.declaration.params.iter().zip(&args) {
            environment.define(&arg.lexeme, arg_value.clone());
        }
        if let Err(e) = interpreter.execute_block(&self.0.declaration.body, &environment) {
            match e {
                InterpreterError::Return(v) => Ok(v),
                e => Err(e),
            }
        } else {
            Ok(RuntimeValue::Nil)
        }
    }
    fn arity(&self) -> usize {
        self.0.declaration.params.len()
    }
}

pub struct BuiltInFunctionStorage {
    name: String,
    args: Vec<String>,
    callable: fn(&Interpreter, Vec<RuntimeValue>) -> Result<RuntimeValue, InterpreterError>,
}
#[derive(Clone)]
pub struct BuiltInFunction(Arc<BuiltInFunctionStorage>);

impl Debug for BuiltInFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BuiltInFunction{{ name: {:?}, args: {:?}, callable: ?? }}",
            self.0.name, self.0.args
        )
    }
}
impl Display for BuiltInFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fun {}({})>", self.0.name, self.0.args.join(", "))
    }
}
impl PartialEq for BuiltInFunction {
    fn eq(&self, other: &Self) -> bool {
        self.0.name == other.0.name
    }
}
impl BuiltInFunction {
    pub fn new(
        name: &str,
        args: Vec<&str>,
        callable: fn(&Interpreter, Vec<RuntimeValue>) -> Result<RuntimeValue, InterpreterError>,
    ) -> Self {
        Self(
            BuiltInFunctionStorage {
                name: name.into(),
                args: args.into_iter().map(str::to_string).collect(),
                callable,
            }
            .into(),
        )
    }
}

impl CallableValue for BuiltInFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue, InterpreterError> {
        (self.0.callable)(interpreter, args)
    }
    fn arity(&self) -> usize {
        self.0.args.len()
    }
}
