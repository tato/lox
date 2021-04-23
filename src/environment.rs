use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::value::LoxValue;

pub struct Environment {
    values: HashMap<String, LoxValue>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(
            Self {
                values: HashMap::new(),
                enclosing: None,
            }
            .into(),
        )
    }
    pub fn new_child(enclosing: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(
            Self {
                values: HashMap::new(),
                enclosing: Some(enclosing),
            }
            .into(),
        )
    }
    pub fn define(&mut self, name: String, value: LoxValue) {
        self.values.insert(name, value);
    }
    pub fn assign(&mut self, name: String, value: LoxValue) -> Option<LoxValue> {
        if self.values.contains_key(&name) {
            self.values.insert(name, value)
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, value)
        } else {
            None
        }
    }
    pub fn get(&self, name: &String) -> Option<LoxValue> {
        let mut value = self.values.get(name).cloned();
        if value.is_none() {
            if let Some(enclosing) = &self.enclosing {
                value = enclosing.borrow().get(name);
            }
        }
        value
    }
}
