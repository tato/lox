use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::{
    interpreter::{Interpreter, InterpreterError},
    token::Token,
};

use super::{CallableValue, RuntimeValue, UserFunction};

#[derive(Debug)]
pub struct ClassDefinitionStorage {
    name: Token,
    superclass: Option<ClassDefinition>,
    methods: HashMap<String, UserFunction>,
}
#[derive(Debug, Clone)]
pub struct ClassDefinition(Arc<ClassDefinitionStorage>);

impl Display for ClassDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<class {}>", self.0.name.lexeme)
    }
}
impl PartialEq for ClassDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.0.name == other.0.name
    }
}
impl CallableValue for ClassDefinition {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue, InterpreterError> {
        let instance = ClassInstance::new(self);
        let initializer = self.find_method("init");
        if let Some(fun) = initializer {
            fun.bind(&instance).call(interpreter, args)?;
        }
        Ok(RuntimeValue::Instance(instance))
    }

    fn arity(&self) -> usize {
        let initializer = self.find_method("init");
        initializer.as_ref().map(CallableValue::arity).unwrap_or(0)
    }
}
impl ClassDefinition {
    pub fn new(
        name: &Token,
        superclass: Option<ClassDefinition>,
        methods: HashMap<String, UserFunction>,
    ) -> Self {
        Self(
            ClassDefinitionStorage {
                name: name.clone(),
                superclass,
                methods,
            }
            .into(),
        )
    }
    pub fn find_method(&self, name: &str) -> Option<UserFunction> {
        let self_method = self.0.methods.get(name).cloned();
        match (&self_method, &self.0.superclass) {
            (Some(_), _) => self_method,
            (None, Some(sc)) => sc.find_method(name),
            (None, None) => None,
        }
    }
}
#[derive(Debug)]
struct ClassInstanceStorage {
    class: Arc<ClassDefinition>,
    fields: Mutex<HashMap<String, RuntimeValue>>,
}
#[derive(Debug, Clone)]
pub struct ClassInstance(Arc<ClassInstanceStorage>);

impl Display for ClassInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "instance {}({})",
            &self.0.class.0.name.lexeme,
            self.0
                .fields
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
        let map: &HashMap<String, RuntimeValue> = &self.0.fields.lock().unwrap();
        let other_map: &HashMap<String, RuntimeValue> = &other.0.fields.lock().unwrap();
        self.0.class == other.0.class && map == other_map
    }
}
impl ClassInstance {
    pub fn new(class: &ClassDefinition) -> Self {
        Self(
            ClassInstanceStorage {
                class: class.clone().into(),
                fields: HashMap::new().into(),
            }
            .into(),
        )
    }
    pub fn get(&self, name: &Token) -> Option<RuntimeValue> {
        let field = self.0.fields.lock().unwrap().get(&name.lexeme).cloned();
        match field {
            Some(_) => field,
            None => self
                .0
                .class
                .find_method(&name.lexeme)
                .map(|it| it.bind(self))
                .map(RuntimeValue::UserFunction),
        }
    }
    pub fn set(&self, name: &Token, value: RuntimeValue) {
        self.0
            .fields
            .lock()
            .unwrap()
            .insert(name.lexeme.clone(), value);
    }
}
