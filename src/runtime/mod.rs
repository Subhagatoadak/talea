// src/runtime/mod.rs

use std::collections::HashMap;

// This line makes the interpreter module public within the runtime module.
pub mod interpreter;

// Represents all possible values in talea
#[derive(Debug, Clone)]
pub enum TaleaValue {
    String(String),
    Null,
}

// The Environment holds all our variable bindings
pub struct Environment {
    store: HashMap<String, TaleaValue>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: TaleaValue) {
        self.store.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<TaleaValue> {
        self.store.get(name).cloned()
    }
}