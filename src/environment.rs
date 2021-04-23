use std::collections::HashMap;

use crate::{token::{Token}, value::LoxValue};

pub struct Environment {
    values: HashMap<String, LoxValue>,
}

impl Environment {
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }
    pub fn define(&mut self, name: String, value: LoxValue) {
        self.values.insert(name, value);
    }
    pub fn get(&self, name: &Token) -> Option<LoxValue> {
        self.values.get(&name.lexeme()).cloned()
    }
}