use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::value::RuntimeValue;

struct EnvironmentStorage {
    values: Mutex<HashMap<String, RuntimeValue>>,
    enclosing: Option<Environment>,
}
#[derive(Clone)]
pub struct Environment(Arc<EnvironmentStorage>);

impl Environment {
    pub fn new() -> Self {
        Self(
            EnvironmentStorage {
                values: HashMap::new().into(),
                enclosing: None,
            }
            .into(),
        )
    }
    pub fn child(&self) -> Self {
        Self(
            EnvironmentStorage {
                values: HashMap::new().into(),
                enclosing: Some(self.clone()),
            }
            .into(),
        )
    }
    pub fn enclosing(&self) -> Option<Environment> {
        self.0.enclosing.clone()
    }
    pub fn define(&self, name: &str, value: RuntimeValue) {
        self.0
            .values
            .lock()
            .unwrap()
            .insert(name.to_string(), value);
    }
    pub fn assign(&self, name: &str, value: RuntimeValue) -> Option<RuntimeValue> {
        let mut values = self.0.values.lock().unwrap();
        if values.contains_key(name) {
            values.insert(name.to_string(), value)
        } else if let Some(enclosing) = &self.0.enclosing {
            enclosing.assign(name, value)
        } else {
            None
        }
    }
    pub fn assign_at(
        &self,
        distance: usize,
        name: &str,
        value: RuntimeValue,
    ) -> Option<RuntimeValue> {
        if distance > 0 {
            self.ancestor(distance)
                .0
                .values
                .lock()
                .unwrap()
                .insert(name.to_string(), value)
        } else {
            self.0
                .values
                .lock()
                .unwrap()
                .insert(name.to_string(), value)
        }
    }
    pub fn get(&self, name: &str) -> Option<RuntimeValue> {
        let mut value = self.0.values.lock().unwrap().get(name).cloned();
        if value.is_none() {
            if let Some(enclosing) = &self.0.enclosing {
                value = enclosing.get(name);
            }
        }
        value
    }
    pub fn get_at(&self, distance: usize, name: &str) -> Option<RuntimeValue> {
        if distance > 0 {
            self.ancestor(distance)
                .0
                .values
                .lock()
                .unwrap()
                .get(name)
                .cloned()
        } else {
            self.0.values.lock().unwrap().get(name).cloned()
        }
    }

    fn ancestor(&self, distance: usize) -> Environment {
        assert!(distance > 0);
        assert!(self.0.enclosing.is_some());

        // yuck
        let mut env = self.0.enclosing.clone();
        for _ in 0..(distance - 1) {
            let some = env.take().unwrap();
            if let Some(enclosing) = some.0.enclosing.as_ref() {
                env = Some(enclosing.clone())
            }
        }
        env.unwrap()
    }
}
