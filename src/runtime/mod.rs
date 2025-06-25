// src/runtime/mod.rs

use std::collections::HashMap;
use crate::lexer::Token;

pub mod interpreter;

#[derive(Debug, Clone)]
pub enum TaleaValue {
    String(String),
    Number(i64),
    List(Vec<TaleaValue>),
    Tuple(Vec<TaleaValue>),
    Unit(Token),
    Null,
}

pub struct Environment {
    store: HashMap<String, TaleaValue>,
}

impl Environment {
    pub fn new() -> Self {
        Environment { store: HashMap::new() }
    }
    pub fn define(&mut self, name: String, value: TaleaValue) {
        self.store.insert(name, value);
    }
    pub fn get(&self, name: &str) -> Option<TaleaValue> {
        self.store.get(name).cloned()
    }
}