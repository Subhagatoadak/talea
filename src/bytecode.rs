// src/bytecode.rs
//
// Bytecode definitions for the Talea VM.
// A `Chunk` contains a constant pool and a sequence of `Instruction`s.
// Chunks can be serialized to / deserialized from `.taleac` files via serde_json.

use serde::{Deserialize, Serialize};

// ────────────────────────────────────────────────────────────
// Serialisable value type (mirrors TaleaValue but without pyo3)
// ────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Constant {
    String(String),
    Number(i64),
    Float(f64),
    Bool(bool),
    Null,
}

// ────────────────────────────────────────────────────────────
// Opcodes
// ────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpCode {
    // ── Stack / variables ───────────────────────────────────
    /// Push constant[index] onto the stack.
    LoadConst(usize),
    /// Push the value of variable `name` onto the stack.
    LoadVar(String),
    /// Pop top of stack and store in variable `name`.
    StoreVar(String),
    /// Index: pop index, pop collection → push collection[index].
    Index,
    /// Build a list of `n` items from the top of the stack.
    BuildList(usize),

    // ── I/O ─────────────────────────────────────────────────
    /// print <top>
    Print,
    /// load file: pop alias-name, pop path → load file, store in alias.
    Load,
    /// load csv: pop alias-name, pop path → load CSV, store in alias.
    LoadCsv,
    /// save: pop destination-path, pop source.
    Save,
    /// define: pop value, pop name-string → store.
    Define,

    // ── NLP ─────────────────────────────────────────────────
    /// tokenize: pop source → push token list, then StoreVar follows.
    Tokenize,
    Sentiment,
    Count,
    Split,
    Join,
    Replace,
    Clean,
    Lowercase,
    Uppercase,
    Stem,
    Normalize,
    Lemmatize,
    /// Tag: pop method-string, pop source → push tagged list.
    Tag,
    Concordance,
    WordCloud,
    Association,
    SemanticSimilarity,
    SemanticSearch,
    KeywordSearch,
    Translate,
    Summarize,

    // ── Statistics ──────────────────────────────────────────
    Regress,
    TTest,
    Anova,
    Shapiro,
    Kolmogorov,
    Mean,
    Median,
    Mode,
    Skewness,
    Kurtosis,
    Insight,
    Doc2Bow,

    // ── Arithmetic ──────────────────────────────────────────
    Add,
    Sub,
    Mul,
    Div,

    // ── Comparison ──────────────────────────────────────────
    CmpEq,
    CmpGt,
    CmpLt,

    // ── Control flow ────────────────────────────────────────
    /// Unconditional jump to instruction index.
    Jump(usize),
    /// Pop condition; if falsy jump to index, else fall through.
    JumpIfFalse(usize),
    /// Marks a position (no-op at runtime, used by compiler for back-patching).
    Label(usize),

    // ── Backend / env ───────────────────────────────────────
    /// `use python | r | java`
    UseBackend(String),
    SetEnv,
    Import,
    Call,
    CallMethod,
    OpenAI,

    // ── Misc ─────────────────────────────────────────────────
    Nop,
    Pop,
}

// ────────────────────────────────────────────────────────────
// Chunk – the bytecode container
// ────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Chunk {
    /// Constant pool: all literal values used by instructions.
    pub constants: Vec<Constant>,
    /// Instruction stream.
    pub instructions: Vec<OpCode>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            constants: Vec::new(),
            instructions: Vec::new(),
        }
    }

    /// Add a constant and return its index.
    pub fn add_constant(&mut self, c: Constant) -> usize {
        // Deduplicate
        if let Some(pos) = self.constants.iter().position(|x| x == &c) {
            return pos;
        }
        let idx = self.constants.len();
        self.constants.push(c);
        idx
    }

    /// Append an instruction and return its index.
    pub fn emit(&mut self, op: OpCode) -> usize {
        let idx = self.instructions.len();
        self.instructions.push(op);
        idx
    }

    /// Overwrite an instruction at `idx` (used for back-patching jumps).
    pub fn patch(&mut self, idx: usize, op: OpCode) {
        self.instructions[idx] = op;
    }

    // ── Serialisation ────────────────────────────────────────

    /// Serialize to JSON bytes (suitable for `.taleac` files).
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }

    /// Deserialize from JSON bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| e.to_string())
    }
}
