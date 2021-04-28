use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::value::RuntimeValue;

pub struct Environment {
    values: Mutex<HashMap<String, RuntimeValue>>,
    enclosing: Option<Arc<Environment>>,
}

impl Environment {
    pub fn new() -> Arc<Self> {
        Arc::new(
            Self {
                values: HashMap::new().into(),
                enclosing: None,
            }
            .into(),
        )
    }
    pub fn new_child(enclosing: Arc<Environment>) -> Arc<Self> {
        Arc::new(
            Self {
                values: HashMap::new().into(),
                enclosing: Some(enclosing),
            }
            .into(),
        )
    }
    pub fn define(&self, name: &str, value: RuntimeValue) {
        self.values.lock().unwrap().insert(name.to_string(), value);
    }
    pub fn assign(&self, name: &str, value: RuntimeValue) -> Option<RuntimeValue> {
        let mut values = self.values.lock().unwrap();
        if values.contains_key(name) {
            values.insert(name.to_string(), value)
        } else if let Some(enclosing) = &self.enclosing {
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
                .values
                .lock()
                .unwrap()
                .insert(name.to_string(), value)
        } else {
            self.values.lock().unwrap().insert(name.to_string(), value)
        }
    }
    pub fn get(&self, name: &str) -> Option<RuntimeValue> {
        let mut value = self.values.lock().unwrap().get(name).cloned();
        if value.is_none() {
            if let Some(enclosing) = &self.enclosing {
                value = enclosing.get(name);
            }
        }
        value
    }
    pub fn get_at(&self, distance: usize, name: &str) -> Option<RuntimeValue> {
        if distance > 0 {
            self.ancestor(distance)
                .values
                .lock()
                .unwrap()
                .get(name)
                .cloned()
        } else {
            self.values.lock().unwrap().get(name).cloned()
        }
    }

    fn ancestor(&self, distance: usize) -> Arc<Environment> {
        assert!(distance > 0);
        assert!(self.enclosing.is_some());

        // yuck
        let mut env = self.enclosing.clone();
        for _ in 0..(distance - 1) {
            let some = env.take().unwrap();
            if let Some(enclosing) = some.enclosing.as_ref() {
                env = Some(enclosing.clone())
            }
        }
        env.unwrap()
    }
}
