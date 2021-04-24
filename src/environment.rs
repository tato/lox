use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::value::LoxValue;

pub struct Environment {
    values: HashMap<String, LoxValue>,
    enclosing: Option<Arc<Mutex<Environment>>>,
}

impl Environment {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(
            Self {
                values: HashMap::new(),
                enclosing: None,
            }
            .into(),
        )
    }
    pub fn new_child(enclosing: Arc<Mutex<Environment>>) -> Arc<Mutex<Self>> {
        Arc::new(
            Self {
                values: HashMap::new(),
                enclosing: Some(enclosing),
            }
            .into(),
        )
    }
    pub fn define(&mut self, name: &str, value: LoxValue) {
        self.values.insert(name.to_string(), value);
    }
    pub fn assign(&mut self, name: &str, value: LoxValue) -> Option<LoxValue> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value)
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.lock().unwrap().assign(name, value)
        } else {
            None
        }
    }
    pub fn get(&self, name: &str) -> Option<LoxValue> {
        let mut value = self.values.get(name).cloned();
        if value.is_none() {
            if let Some(enclosing) = &self.enclosing {
                value = enclosing.lock().unwrap().get(name);
            }
        }
        value
    }
}
