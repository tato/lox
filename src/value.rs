use std::{collections::HashMap, fmt::{Debug, Display}, rc::Rc, sync::{Arc, Mutex}};

use crate::{
    ast::Stmt,
    environment::Environment,
    interpreter::{Interpreter, InterpreterError},
    token::Token,
};

pub trait CallableValue {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue, InterpreterError>;
    fn arity(&self) -> usize;
}

#[derive(Clone)]
pub struct BuiltInFunction {
    name: String,
    args: Vec<String>,
    callable: fn(&Interpreter, Vec<RuntimeValue>) -> Result<RuntimeValue, InterpreterError>,
}
impl Debug for BuiltInFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BuiltInFunction{{ name: {:?}, args: {:?}, callable: ?? }}",
            self.name, self.args
        )
    }
}
impl Display for BuiltInFunction {
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
        callable: fn(&Interpreter, Vec<RuntimeValue>) -> Result<RuntimeValue, InterpreterError>,
    ) -> Self {
        Self {
            name: name.into(),
            args: args.into_iter().map(str::to_string).collect(),
            callable,
        }
    }
}
impl CallableValue for BuiltInFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue, InterpreterError> {
        (self.callable)(interpreter, args)
    }
    fn arity(&self) -> usize {
        self.args.len()
    }
}

#[derive(Clone)]
pub struct UserFunction {
    name: Token,
    args: Vec<Token>,
    body: Vec<Stmt>,
    closure: Arc<Mutex<Environment>>,
}
impl Debug for UserFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UserFunction{{ name: {:?}, args: {:?}, body: {:?}, closure: ?? }}",
            self.name, self.args, self.body
        )
    }
}
impl Display for UserFunction {
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
        self.name == other.name
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
impl CallableValue for UserFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue, InterpreterError> {
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
            Ok(RuntimeValue::Nil)
        }
    }
    fn arity(&self) -> usize {
        self.args.len()
    }
}


#[derive(Debug, Clone)]
pub struct ClassDefinition {
    name: Token,
}
impl Display for ClassDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<class {}>", self.name.lexeme)
    }
}
impl PartialEq for ClassDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl CallableValue for ClassDefinition {
    fn call(
        &self,
        _: &mut Interpreter,
        _: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue, InterpreterError> {
        let instance = ClassInstance::new(self);
        Ok(RuntimeValue::Instance(instance.into()))
    }

    fn arity(&self) -> usize {
        0
    }
}
impl ClassDefinition {
    pub fn new(name: &Token) -> Self {
        Self { name: name.clone() }
    }
}

#[derive(Clone, Debug)]
pub struct ClassInstance {
    class: Box<ClassDefinition>, // TODO!
    fields: Rc<HashMap<String, RuntimeValue>>, // TODO! IDK WHATEVER XD
}
impl Display for ClassInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
impl PartialEq for ClassInstance {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}
impl ClassInstance {
    pub fn new(class: &ClassDefinition) -> Self {
        Self {
            class: class.clone().into(),
            fields: HashMap::new().into(),
        }
    }
    pub fn get(&self, name: &Token) -> Option<RuntimeValue> {
        self.fields.get(&name.lexeme).cloned()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Bool(bool),
    Float(f64),
    Str(String),
    BuiltInFunction(Box<BuiltInFunction>),
    UserFunction(Box<UserFunction>),
    Class(Box<ClassDefinition>),
    Instance(Box<ClassInstance>),
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
            RuntimeValue::BuiltInFunction(x) => Some(x.as_ref()),
            RuntimeValue::UserFunction(x) => Some(x.as_ref()),
            RuntimeValue::Class(x) => Some(x.as_ref()),
            _ => None,
        }
    }
}
