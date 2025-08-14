// src/runtime/mod.rs

use crate::lexer::Token;
#[cfg(feature = "r")]
use extendr_api::prelude::Robj;
use pyo3::{PyObject, Python};
use serde_json::{Value as JsonValue, json};
use std::collections::HashMap;
use std::fs;

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

fn talea_value_to_json(value: &TaleaValue) -> Result<JsonValue, String> {
    match value {
        TaleaValue::String(s) => Ok(json!({"type": "String", "value": s})),
        TaleaValue::Number(n) => Ok(json!({"type": "Number", "value": n})),
        TaleaValue::Float(f) => Ok(json!({"type": "Float", "value": f})),
        TaleaValue::List(list) => {
            let items = list
                .iter()
                .map(|v| talea_value_to_json(v))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(json!({"type": "List", "value": items}))
        }
        TaleaValue::Tuple(list) => {
            let items = list
                .iter()
                .map(|v| talea_value_to_json(v))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(json!({"type": "Tuple", "value": items}))
        }
        TaleaValue::Table(rows) => {
            let mut arr = Vec::new();
            for row in rows {
                let mut obj = serde_json::Map::new();
                for (k, v) in row {
                    obj.insert(k.clone(), talea_value_to_json(v)?);
                }
                arr.push(JsonValue::Object(obj));
            }
            Ok(json!({"type": "Table", "value": arr}))
        }
        TaleaValue::Null => Ok(json!({"type": "Null"})),
        TaleaValue::PyObject(_) => Err("Cannot serialize Python objects".to_string()),
        #[cfg(feature = "r")]
        TaleaValue::RObject(_) => Err("Cannot serialize R objects".to_string()),
        TaleaValue::Unit(_) => Err("Cannot serialize Unit values".to_string()),
    }
}

fn json_to_talea_value(value: &JsonValue) -> Result<TaleaValue, String> {
    let obj = value
        .as_object()
        .ok_or_else(|| "Invalid Talea object".to_string())?;
    let typ = obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Talea object missing type".to_string())?;
    match typ {
        "String" => Ok(TaleaValue::String(
            obj.get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Invalid String value".to_string())?
                .to_string(),
        )),
        "Number" => Ok(TaleaValue::Number(
            obj.get("value")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| "Invalid Number value".to_string())?,
        )),
        "Float" => Ok(TaleaValue::Float(
            obj.get("value")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| "Invalid Float value".to_string())?,
        )),
        "List" => {
            let arr = obj
                .get("value")
                .and_then(|v| v.as_array())
                .ok_or_else(|| "Invalid List value".to_string())?;
            let mut out = Vec::new();
            for item in arr {
                out.push(json_to_talea_value(item)?);
            }
            Ok(TaleaValue::List(out))
        }
        "Tuple" => {
            let arr = obj
                .get("value")
                .and_then(|v| v.as_array())
                .ok_or_else(|| "Invalid Tuple value".to_string())?;
            let mut out = Vec::new();
            for item in arr {
                out.push(json_to_talea_value(item)?);
            }
            Ok(TaleaValue::Tuple(out))
        }
        "Table" => {
            let arr = obj
                .get("value")
                .and_then(|v| v.as_array())
                .ok_or_else(|| "Invalid Table value".to_string())?;
            let mut rows = Vec::new();
            for item in arr {
                let map_obj = item
                    .as_object()
                    .ok_or_else(|| "Table rows must be objects".to_string())?;
                let mut row = HashMap::new();
                for (k, v) in map_obj {
                    row.insert(k.clone(), json_to_talea_value(v)?);
                }
                rows.push(row);
            }
            Ok(TaleaValue::Table(rows))
        }
        "Null" => Ok(TaleaValue::Null),
        other => Err(format!("Unknown Talea type '{}'", other)),
    }
}

pub fn save_talea_object(value: &TaleaValue, path: &str) -> Result<(), String> {
    let json = talea_value_to_json(value)?;
    let serialized = serde_json::to_string(&json).map_err(|e| e.to_string())?;
    fs::write(path, serialized).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_talea_object(path: &str) -> Result<TaleaValue, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let json: JsonValue = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    json_to_talea_value(&json)
}
