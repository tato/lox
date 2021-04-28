use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::value::RuntimeValue;

pub struct Environment {
    values: HashMap<String, RuntimeValue>,
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
    pub fn define(&mut self, name: &str, value: RuntimeValue) {
        self.values.insert(name.to_string(), value);
    }
    pub fn assign(&mut self, name: &str, value: RuntimeValue) -> Option<RuntimeValue> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value)
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.lock().unwrap().assign(name, value)
        } else {
            None
        }
    }
    pub fn assign_at(
        &mut self,
        distance: usize,
        name: &str,
        value: RuntimeValue,
    ) -> Option<RuntimeValue> {
        if distance > 0 {
            self.ancestor(distance)
                .lock()
                .unwrap()
                .values
                .insert(name.to_string(), value)
        } else {
            self.values.insert(name.to_string(), value)
        }
    }
    pub fn get(&self, name: &str) -> Option<RuntimeValue> {
        let mut value = self.values.get(name).cloned();
        if value.is_none() {
            if let Some(enclosing) = &self.enclosing {
                value = enclosing.lock().unwrap().get(name);
            }
        }
        value
    }
    pub fn get_at(&self, distance: usize, name: &str) -> Option<RuntimeValue> {
        if distance > 0 {
            self.ancestor(distance)
                .lock()
                .unwrap()
                .values
                .get(name)
                .cloned()
        } else {
            self.values.get(name).cloned()
        }
    }

    fn ancestor(&self, distance: usize) -> Arc<Mutex<Environment>> {
        assert!(distance > 0);
        assert!(self.enclosing.is_some());

        // yuck
        let mut env = self.enclosing.clone();
        for _ in 0..(distance - 1) {
            let some = env.take().unwrap();
            let locked = some.lock().unwrap();
            if let Some(enclosing) = locked.enclosing.as_ref() {
                env = Some(enclosing.clone())
            }
        }
        env.unwrap()
    }
}
