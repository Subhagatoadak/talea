// src/runtime/mod.rs

use crate::lexer::Token;
#[cfg(feature = "r")]
use extendr_api::prelude::Robj;
use pyo3::{PyObject, Python};
use std::collections::HashMap;

pub mod interpreter;

#[derive(Debug)]
pub enum TaleaValue {
    String(String),
    Number(i64),
    Float(f64),
    List(Vec<TaleaValue>),
    Tuple(Vec<TaleaValue>),
    Table(Vec<HashMap<String, TaleaValue>>),
    PyObject(PyObject),
    #[cfg(feature = "r")]
    RObject(Robj),
    Unit(Token),
    Null,
}

impl PartialEq for TaleaValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TaleaValue::String(a), TaleaValue::String(b)) => a == b,
            (TaleaValue::Number(a), TaleaValue::Number(b)) => a == b,
            (TaleaValue::Float(a), TaleaValue::Float(b)) => a == b,
            (TaleaValue::List(a), TaleaValue::List(b)) => a == b,
            (TaleaValue::Tuple(a), TaleaValue::Tuple(b)) => a == b,
            (TaleaValue::Table(a), TaleaValue::Table(b)) => a == b,
            (TaleaValue::Unit(a), TaleaValue::Unit(b)) => a == b,
            (TaleaValue::Null, TaleaValue::Null) => true,
            #[cfg(feature = "r")]
            (TaleaValue::RObject(_), TaleaValue::RObject(_)) => false,
            (TaleaValue::PyObject(_), TaleaValue::PyObject(_)) => false,
            _ => false,
        }
    }
}

impl Clone for TaleaValue {
    fn clone(&self) -> Self {
        match self {
            TaleaValue::String(s) => TaleaValue::String(s.clone()),
            TaleaValue::Number(n) => TaleaValue::Number(*n),
            TaleaValue::Float(f) => TaleaValue::Float(*f),
            TaleaValue::List(l) => TaleaValue::List(l.clone()),
            TaleaValue::Tuple(t) => TaleaValue::Tuple(t.clone()),
            TaleaValue::Table(rows) => TaleaValue::Table(rows.clone()),
            TaleaValue::PyObject(obj) => {
                Python::with_gil(|py| TaleaValue::PyObject(obj.clone_ref(py)))
            }
            #[cfg(feature = "r")]
            TaleaValue::RObject(obj) => TaleaValue::RObject(obj.clone()),
            TaleaValue::Unit(t) => TaleaValue::Unit(t.clone()),
            TaleaValue::Null => TaleaValue::Null,
        }
    }
}

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
