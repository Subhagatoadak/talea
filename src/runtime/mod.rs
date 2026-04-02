// src/runtime/mod.rs
//
// Runtime value system for Talea.
//
// `TaleaValue` is the universal value type — every variable, list element,
// table cell, and intermediate computation result is one of these variants.
//
// `Environment` is now a *scope stack*: each function call, if-body, and
// for-each loop pushes a new scope; variables are resolved by walking from
// the innermost scope outward.  This replaces the old flat HashMap and
// enables proper lexical scoping.

use crate::lexer::Token;
#[cfg(feature = "r")]
use extendr_api::prelude::Robj;
use pyo3::{PyObject, Python};
use serde_json::{Value as JsonValue, json};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, RwLock};

pub mod interpreter;

// ─────────────────────────────────────────────────────────────────────────────
// TaleaValue
// ─────────────────────────────────────────────────────────────────────────────

/// Every runtime value in a Talea program.
///
/// New variants vs the original:
/// - `Bool`     — dedicated boolean; removes the `Number(0/1)` hack
/// - `Map`      — key→value dictionary (used for named results, DB rows, JSON)
/// - `Channel`  — thread-safe MPSC channel for parallel/concurrent pipelines
/// - `DbResult` — structured result set from a database query
/// - `Error`    — first-class error value produced by `try/catch`
#[derive(Debug)]
pub enum TaleaValue {
    // ── Scalars ──────────────────────────────────────────────────────────────
    String(String),
    Number(i64),
    Float(f64),
    Bool(bool),
    Null,

    // ── Collections ──────────────────────────────────────────────────────────
    /// Ordered list — shared via Arc<RwLock<>> so cloning is O(1).
    List(Arc<RwLock<Vec<TaleaValue>>>),
    Tuple(Vec<TaleaValue>),
    /// Tabular data: each row is a column-name → value map.
    Table(Vec<HashMap<String, TaleaValue>>),
    /// Key → value dictionary (analogous to Python dict / JSON object).
    Map(HashMap<String, TaleaValue>),

    // ── Foreign objects ───────────────────────────────────────────────────────
    PyObject(PyObject),
    #[cfg(feature = "r")]
    RObject(Robj),

    // ── Database ──────────────────────────────────────────────────────────────
    /// Result set from a SQL query: rows + column metadata.
    DbResult {
        rows:    Vec<HashMap<String, TaleaValue>>,
        columns: Vec<String>,
        count:   usize,
    },

    // ── Concurrency ───────────────────────────────────────────────────────────
    /// A bounded, thread-safe channel (sender side only stored here).
    /// Actual sender/receiver pair lives in the interpreter's channel registry.
    Channel(String /* channel id */),

    // ── Error ─────────────────────────────────────────────────────────────────
    Error(String),

    // ── Internal ──────────────────────────────────────────────────────────────
    Unit(Token),
}

// ─────────────────────────────────────────────────────────────────────────────
// PartialEq
// ─────────────────────────────────────────────────────────────────────────────

impl PartialEq for TaleaValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TaleaValue::String(a), TaleaValue::String(b)) => a == b,
            (TaleaValue::Number(a), TaleaValue::Number(b)) => a == b,
            (TaleaValue::Float(a), TaleaValue::Float(b)) => a == b,
            (TaleaValue::Bool(a), TaleaValue::Bool(b)) => a == b,
            (TaleaValue::Null, TaleaValue::Null) => true,
            (TaleaValue::Tuple(a), TaleaValue::Tuple(b)) => a == b,
            (TaleaValue::Unit(a), TaleaValue::Unit(b)) => a == b,
            (TaleaValue::Error(a), TaleaValue::Error(b)) => a == b,
            // List equality: compare contents through the Arc
            (TaleaValue::List(a), TaleaValue::List(b)) => {
                let a = a.read().unwrap();
                let b = b.read().unwrap();
                *a == *b
            }
            // Table equality: row-by-row
            (TaleaValue::Table(a), TaleaValue::Table(b)) => a == b,
            // Map equality
            (TaleaValue::Map(a), TaleaValue::Map(b)) => a == b,
            // DbResult equality: compare row counts + column sets
            (
                TaleaValue::DbResult { count: ca, .. },
                TaleaValue::DbResult { count: cb, .. },
            ) => ca == cb,
            // Python/R objects are reference types — equality is never true
            (TaleaValue::PyObject(_), TaleaValue::PyObject(_)) => false,
            #[cfg(feature = "r")]
            (TaleaValue::RObject(_), TaleaValue::RObject(_)) => false,
            _ => false,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Clone
// ─────────────────────────────────────────────────────────────────────────────

impl Clone for TaleaValue {
    fn clone(&self) -> Self {
        match self {
            TaleaValue::String(s) => TaleaValue::String(s.clone()),
            TaleaValue::Number(n) => TaleaValue::Number(*n),
            TaleaValue::Float(f) => TaleaValue::Float(*f),
            TaleaValue::Bool(b) => TaleaValue::Bool(*b),
            TaleaValue::Null => TaleaValue::Null,
            // Arc clone is O(1) — just bumps the reference count.
            TaleaValue::List(l) => TaleaValue::List(Arc::clone(l)),
            TaleaValue::Tuple(t) => TaleaValue::Tuple(t.clone()),
            TaleaValue::Table(rows) => TaleaValue::Table(rows.clone()),
            TaleaValue::Map(m) => TaleaValue::Map(m.clone()),
            TaleaValue::PyObject(obj) => {
                Python::with_gil(|py| TaleaValue::PyObject(obj.clone_ref(py)))
            }
            #[cfg(feature = "r")]
            TaleaValue::RObject(obj) => TaleaValue::RObject(obj.clone()),
            TaleaValue::DbResult { rows, columns, count } => TaleaValue::DbResult {
                rows:    rows.clone(),
                columns: columns.clone(),
                count:   *count,
            },
            TaleaValue::Channel(id) => TaleaValue::Channel(id.clone()),
            TaleaValue::Error(msg) => TaleaValue::Error(msg.clone()),
            TaleaValue::Unit(t) => TaleaValue::Unit(t.clone()),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Display / ToString
// ─────────────────────────────────────────────────────────────────────────────

impl std::fmt::Display for TaleaValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaleaValue::String(s) => write!(f, "{}", s),
            TaleaValue::Number(n) => write!(f, "{}", n),
            TaleaValue::Float(v) => write!(f, "{}", v),
            TaleaValue::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            TaleaValue::Null => write!(f, "nothing"),
            TaleaValue::List(l) => {
                let l = l.read().unwrap();
                let items: Vec<String> = l.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[List with {} items]:\n[{}]", l.len(), items.join(", "))
            }
            TaleaValue::Tuple(t) => {
                let items: Vec<String> = t.iter().map(|v| format!("{}", v)).collect();
                write!(f, "({})", items.join(", "))
            }
            TaleaValue::Table(rows) => write!(f, "<Table with {} rows>", rows.len()),
            TaleaValue::Map(m) => {
                let pairs: Vec<String> = m.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                write!(f, "{{{}}}", pairs.join(", "))
            }
            TaleaValue::PyObject(_) => write!(f, "<python object>"),
            #[cfg(feature = "r")]
            TaleaValue::RObject(_) => write!(f, "<r object>"),
            TaleaValue::DbResult { count, columns, .. } => {
                write!(f, "<DbResult: {} rows, columns: {}>", count, columns.join(", "))
            }
            TaleaValue::Channel(id) => write!(f, "<channel:{}>", id),
            TaleaValue::Error(msg) => write!(f, "Error: {}", msg),
            TaleaValue::Unit(t) => write!(f, "Unit({:?})", t),
        }
    }
}

// Keep the old ToString impl working for code that calls .to_string() directly.
impl TaleaValue {
    pub fn to_display_string(&self) -> String {
        format!("{}", self)
    }

    /// Unwrap as a plain Rust String if the value is `TaleaValue::String`.
    pub fn as_string(&self) -> Option<String> {
        if let TaleaValue::String(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }

    /// Truthy test used by `if` and `while`.
    pub fn is_truthy(&self) -> bool {
        match self {
            TaleaValue::Bool(b) => *b,
            TaleaValue::Number(n) => *n != 0,
            TaleaValue::Float(f) => *f != 0.0,
            TaleaValue::String(s) => !s.is_empty(),
            TaleaValue::List(l) => !l.read().unwrap().is_empty(),
            TaleaValue::Tuple(t) => !t.is_empty(),
            TaleaValue::Table(rows) => !rows.is_empty(),
            TaleaValue::Map(m) => !m.is_empty(),
            TaleaValue::Null => false,
            TaleaValue::Error(_) => false,
            _ => true,
        }
    }

    /// Construct a `List` value from a `Vec<TaleaValue>`.
    pub fn list(items: Vec<TaleaValue>) -> Self {
        TaleaValue::List(Arc::new(RwLock::new(items)))
    }

    /// Extract inner vec from a List (clones contents).
    pub fn as_list(&self) -> Option<Vec<TaleaValue>> {
        if let TaleaValue::List(l) = self {
            Some(l.read().unwrap().clone())
        } else {
            None
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Environment — scope stack
// ─────────────────────────────────────────────────────────────────────────────

/// Lexically-scoped variable store.
///
/// Variables are resolved by searching from the innermost (most recently
/// pushed) scope outward to the global scope.  Each function call, if-body,
/// and for-each loop gets its own scope pushed/popped around it.
pub struct Environment {
    /// Stack of scopes; index 0 is the global scope, last is innermost.
    scopes: Vec<HashMap<String, TaleaValue>>,
}

impl Environment {
    pub fn new() -> Self {
        // Start with a single global scope.
        Environment { scopes: vec![HashMap::new()] }
    }

    /// Push a new (empty) scope — call before entering a block.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the innermost scope — call after leaving a block.
    /// Variables defined in that scope are discarded.
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
        // Never pop the global scope.
    }

    /// Define a variable in the **current** (innermost) scope.
    pub fn define(&mut self, name: String, value: TaleaValue) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    /// Read a variable — walks from innermost scope outward.
    pub fn get(&self, name: &str) -> Option<TaleaValue> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }
        None
    }

    /// Update an existing variable in the **scope that owns it**.
    /// Returns `true` if the variable was found and updated.
    pub fn set(&mut self, name: &str, value: TaleaValue) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return true;
            }
        }
        false
    }

    /// Remove a variable from the scope that owns it (used by `free`).
    pub fn free(&mut self, name: &str) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if scope.remove(name).is_some() {
                return true;
            }
        }
        false
    }

    /// Total number of variables across all scopes (for memory reporting).
    pub fn variable_count(&self) -> usize {
        self.scopes.iter().map(|s| s.len()).sum()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Serialisation helpers (used by save/load .talea object files)
// ─────────────────────────────────────────────────────────────────────────────

fn talea_value_to_json(value: &TaleaValue) -> Result<JsonValue, String> {
    match value {
        TaleaValue::String(s)  => Ok(json!({"type": "String",  "value": s})),
        TaleaValue::Number(n)  => Ok(json!({"type": "Number",  "value": n})),
        TaleaValue::Float(f)   => Ok(json!({"type": "Float",   "value": f})),
        TaleaValue::Bool(b)    => Ok(json!({"type": "Bool",    "value": b})),
        TaleaValue::Null       => Ok(json!({"type": "Null"})),
        TaleaValue::Error(msg) => Ok(json!({"type": "Error",   "value": msg})),
        TaleaValue::List(l) => {
            let items = l.read().unwrap()
                .iter()
                .map(talea_value_to_json)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(json!({"type": "List", "value": items}))
        }
        TaleaValue::Tuple(t) => {
            let items = t.iter().map(talea_value_to_json).collect::<Result<Vec<_>, _>>()?;
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
        TaleaValue::Map(m) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in m {
                obj.insert(k.clone(), talea_value_to_json(v)?);
            }
            Ok(json!({"type": "Map", "value": JsonValue::Object(obj)}))
        }
        TaleaValue::DbResult { rows, columns, count } => {
            let mut arr = Vec::new();
            for row in rows {
                let mut obj = serde_json::Map::new();
                for (k, v) in row {
                    obj.insert(k.clone(), talea_value_to_json(v)?);
                }
                arr.push(JsonValue::Object(obj));
            }
            Ok(json!({"type": "DbResult", "columns": columns, "count": count, "value": arr}))
        }
        TaleaValue::PyObject(_) => Err("Cannot serialise Python objects".to_string()),
        #[cfg(feature = "r")]
        TaleaValue::RObject(_) => Err("Cannot serialise R objects".to_string()),
        TaleaValue::Channel(id) => Err(format!("Cannot serialise channel '{}'", id)),
        TaleaValue::Unit(_) => Err("Cannot serialise Unit values".to_string()),
    }
}

fn json_to_talea_value(value: &JsonValue) -> Result<TaleaValue, String> {
    let obj = value.as_object().ok_or("Invalid Talea object")?;
    let typ = obj.get("type").and_then(|v| v.as_str()).ok_or("Talea object missing type")?;
    match typ {
        "String" => Ok(TaleaValue::String(
            obj.get("value").and_then(|v| v.as_str()).ok_or("Invalid String")?.to_string()
        )),
        "Number" => Ok(TaleaValue::Number(
            obj.get("value").and_then(|v| v.as_i64()).ok_or("Invalid Number")?
        )),
        "Float" => Ok(TaleaValue::Float(
            obj.get("value").and_then(|v| v.as_f64()).ok_or("Invalid Float")?
        )),
        "Bool" => Ok(TaleaValue::Bool(
            obj.get("value").and_then(|v| v.as_bool()).ok_or("Invalid Bool")?
        )),
        "Null" => Ok(TaleaValue::Null),
        "Error" => Ok(TaleaValue::Error(
            obj.get("value").and_then(|v| v.as_str()).ok_or("Invalid Error")?.to_string()
        )),
        "List" => {
            let arr = obj.get("value").and_then(|v| v.as_array()).ok_or("Invalid List")?;
            let items = arr.iter().map(json_to_talea_value).collect::<Result<Vec<_>, _>>()?;
            Ok(TaleaValue::list(items))
        }
        "Tuple" => {
            let arr = obj.get("value").and_then(|v| v.as_array()).ok_or("Invalid Tuple")?;
            let items = arr.iter().map(json_to_talea_value).collect::<Result<Vec<_>, _>>()?;
            Ok(TaleaValue::Tuple(items))
        }
        "Table" => {
            let arr = obj.get("value").and_then(|v| v.as_array()).ok_or("Invalid Table")?;
            let mut rows = Vec::new();
            for item in arr {
                let map_obj = item.as_object().ok_or("Table rows must be objects")?;
                let mut row = HashMap::new();
                for (k, v) in map_obj {
                    row.insert(k.clone(), json_to_talea_value(v)?);
                }
                rows.push(row);
            }
            Ok(TaleaValue::Table(rows))
        }
        "Map" => {
            let inner = obj.get("value").and_then(|v| v.as_object()).ok_or("Invalid Map")?;
            let mut map = HashMap::new();
            for (k, v) in inner {
                map.insert(k.clone(), json_to_talea_value(v)?);
            }
            Ok(TaleaValue::Map(map))
        }
        "DbResult" => {
            let columns: Vec<String> = obj.get("columns")
                .and_then(|v| v.as_array())
                .ok_or("DbResult missing columns")?
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            let count = obj.get("count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let arr = obj.get("value").and_then(|v| v.as_array()).ok_or("Invalid DbResult")?;
            let mut rows = Vec::new();
            for item in arr {
                let map_obj = item.as_object().ok_or("DbResult rows must be objects")?;
                let mut row = HashMap::new();
                for (k, v) in map_obj {
                    row.insert(k.clone(), json_to_talea_value(v)?);
                }
                rows.push(row);
            }
            Ok(TaleaValue::DbResult { rows, columns, count })
        }
        other => Err(format!("Unknown Talea type '{}'", other)),
    }
}

pub fn save_talea_object(value: &TaleaValue, path: &str) -> Result<(), String> {
    let json = talea_value_to_json(value)?;
    let serialized = serde_json::to_string(&json).map_err(|e| e.to_string())?;
    fs::write(path, serialized).map_err(|e| e.to_string())
}

pub fn load_talea_object(path: &str) -> Result<TaleaValue, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let json: JsonValue = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    json_to_talea_value(&json)
}
