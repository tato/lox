use std::{
    collections::{HashMap},
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
};

use crate::{ast::{FunctionStmt, Stmt}, environment::Environment, interpreter::{Interpreter, InterpreterError}, token::Token};

pub trait CallableValue {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue, InterpreterError>;
    fn arity(&self) -> usize;
}

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
    declaration: FunctionStmt,
    closure: Arc<Environment>,
}
impl Debug for UserFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UserFunction{{ name: {:?}, args: {:?}, body: {:?}, closure: ?? }}",
            self.declaration.name, self.declaration.params, self.declaration.body
        )
    }
}
impl Display for UserFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<fun {}({})>",
            self.declaration.name.lexeme,
            self.declaration.params
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
        self.declaration.name == other.declaration.name
    }
}
impl UserFunction {
    pub fn new(fun: &FunctionStmt, closure: Arc<Environment>) -> Self {
        Self {
            declaration: fun.clone(),
            closure,
        }
    }
    pub fn bind(&self, instance: &ClassInstance) -> UserFunction {
        let environment = Environment::new_child(self.closure.clone());
        environment.define("this", /*instance.clone()*/ todo!());
        UserFunction::new(&self.declaration, environment)
    }
}
impl CallableValue for UserFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue, InterpreterError> {
        let environment = Environment::new_child(self.closure.clone());
        for (arg, arg_value) in self.declaration.params.iter().zip(&args) {
            environment.define(&arg.lexeme, arg_value.clone());
        }
        if let Err(e) = interpreter.execute_block(&self.declaration.body, environment) {
            match e {
                InterpreterError::Return(v) => Ok(v),
                e => Err(e),
            }
        } else {
            Ok(RuntimeValue::Nil)
        }
    }
    fn arity(&self) -> usize {
        self.declaration.params.len()
    }
}

#[derive(Debug, Clone)]
pub struct ClassDefinition {
    name: Token,
    methods: HashMap<String, Arc<UserFunction>>,
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
    pub fn new(name: &Token, methods: HashMap<String, Arc<UserFunction>>) -> Self {
        Self { name: name.clone(), methods }
    }
    pub fn find_method(&self, name: &str) -> Option<Arc<UserFunction>> {
        self.methods.get(name).cloned()
    }
}

#[derive(Debug)]
pub struct ClassInstance {
    class: Arc<ClassDefinition>,
    fields: Mutex<HashMap<String, RuntimeValue>>,
}
impl Display for ClassInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "instance {}({})",
            &self.class.name.lexeme,
            self.fields
                .lock()
                .unwrap()
                .keys()
                .cloned()
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
impl PartialEq for ClassInstance {
    fn eq(&self, other: &Self) -> bool {
        let map: &HashMap<String, RuntimeValue> = &self.fields.lock().unwrap();
        let other_map: &HashMap<String, RuntimeValue> = &other.fields.lock().unwrap();
        self.class == other.class && map == other_map
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
        let field = self.fields.lock().unwrap().get(&name.lexeme).cloned();
        match field {
            Some(_) => field,
            None => {
                // self.class
                //     .find_method(&name.lexeme)
                //     .map(|it| it.bind(self))
                //     .map(RuntimeValue::UserFunction)
                todo!()
            }
        }
    }
    pub fn set(&self, name: &Token, value: RuntimeValue) {
        self.fields
            .lock()
            .unwrap()
            .insert(name.lexeme.clone(), value);
    }
}



#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Bool(bool),
    Float(f64),
    Str(Arc<str>),
    BuiltInFunction(Arc<BuiltInFunction>),
    UserFunction(Arc<UserFunction>),
    Class(Arc<ClassDefinition>),
    Instance(Arc<ClassInstance>),
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
