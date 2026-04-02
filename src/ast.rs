// src/ast.rs
//
// Abstract Syntax Tree for the Talea language.
//
// Every syntactic construct in a .talea program is represented as a `Statement`
// or an `Expression`.  The parser produces `Vec<Statement>` which is then
// consumed by either the tree-walking interpreter or the bytecode compiler.

use crate::lexer::Token;

// ─────────────────────────────────────────────────────────────────────────────
// Expression
// ─────────────────────────────────────────────────────────────────────────────

/// A value-producing syntactic form.
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Identifier(String),
    StringLiteral(String),
    Number(i64),
    Float(f64),
    Bool(bool),
    Null,
    Unit(Token),
    ListLiteral(Vec<Expression>),
    MapLiteral(Vec<(String, Expression)>),
    Index(Box<Expression>, Box<Expression>),
    /// `left op right` — produces a Bool (or Number 0/1 for the VM)
    Comparison(Box<Expression>, ComparisonOp, Box<Expression>),
    /// `not expr`
    Not(Box<Expression>),
    /// `expr and expr` / `expr or expr`
    Logical(Box<Expression>, LogicalOp, Box<Expression>),
}

// ─────────────────────────────────────────────────────────────────────────────
// Operators
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Clone)]
pub enum ArithmeticOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ComparisonOp {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LogicalOp {
    And,
    Or,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FilterCondition {
    Containing(Expression),
    StartingWith(Expression),
    EndingWith(Expression),
    Matching(Expression), // regex match
}

// ─────────────────────────────────────────────────────────────────────────────
// Backend
// ─────────────────────────────────────────────────────────────────────────────

/// Pluggable execution backend for heavy computation or FFI.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Backend {
    Python,
    R,
    Java,
    Cpp,  // NEW: C++ via cxx bridge
}

// ─────────────────────────────────────────────────────────────────────────────
// Statement
// ─────────────────────────────────────────────────────────────────────────────

/// A single executable instruction in a Talea program.
#[derive(Debug, PartialEq)]
pub enum Statement {
    // ── Backend selection ────────────────────────────────────────────────────
    Use(Backend),

    // ── Help ─────────────────────────────────────────────────────────────────
    Help,

    // ── Bare expression (REPL / print shorthand) ─────────────────────────────
    Expression(Expression),

    // ── Variables ────────────────────────────────────────────────────────────
    /// `let x be 5`  /  `define x as 5`
    Define { name: Expression, value: Expression },

    /// `set environment variable NAME to "value"`
    SetEnv { name: String, value: Expression },

    /// `free my_variable`  — drop the variable from the current scope
    Free { name: Expression },

    // ── I/O ──────────────────────────────────────────────────────────────────
    /// `load "file.txt" as article`
    Load { source: Expression, alias: Expression },

    /// `load csv "data.csv" as table`
    LoadCsv { path: Expression, alias: Expression },

    /// `save result to "output.txt"`
    Save { source: Expression, destination: Expression },

    /// `print x`
    Print(Expression),

    // ── Text processing ───────────────────────────────────────────────────────
    Tokenize   { source: Expression, destination: Expression },
    Doc2Bow    { source: Expression, destination: Expression },
    Split      { unit: Expression, source: Expression, destination: Expression },
    Join       { unit: Expression, source: Expression, separator: Expression, destination: Expression },
    Replace    { pattern: Expression, replacement: Expression, source: Expression, destination: Expression },
    Clean      { source: Expression, destination: Expression },
    Lowercase  { source: Expression, destination: Expression },
    Uppercase  { source: Expression, destination: Expression },
    Stem       { source: Expression, destination: Expression },
    Normalize  { source: Expression, destination: Expression },
    Lemmatize  { source: Expression, destination: Expression },

    // ── NLP analysis ──────────────────────────────────────────────────────────
    Count       { unit: Expression, source: Expression, destination: Expression },
    Concordance { word: Expression, source: Expression, destination: Expression },
    Tag         { source: Expression, method: Expression, destination: Expression },
    Translate   { source: Expression, language: Expression, destination: Expression },
    Sentiment   { source: Expression, destination: Expression },
    Summarize   { source: Expression, destination: Expression },
    Insight     { source: Expression, destination: Expression },
    WordCloud   { source: Expression, destination: Expression },
    WordAssociation { source: Expression, destination: Expression },
    SemanticSimilarity { text1: Expression, text2: Expression, destination: Expression },
    SemanticSearch  { query: Expression, source: Expression, destination: Expression },
    KeywordSearch   { query: Expression, source: Expression, destination: Expression },

    // ── Statistics ────────────────────────────────────────────────────────────
    Compare        { left: Expression, right: Expression, destination: Expression },
    Average        { source: Expression, destination: Expression },
    CheckNormality { source: Expression, destination: Expression },
    Shapiro    { column: Expression, data: Expression, destination: Expression },
    Kolmogorov { column: Expression, data: Expression, destination: Expression },
    Mean       { column: Expression, source: Expression, destination: Expression },
    Median     { column: Expression, source: Expression, destination: Expression },
    Mode       { column: Expression, source: Expression, destination: Expression },
    Skewness   { column: Expression, source: Expression, destination: Expression },
    Kurtosis   { column: Expression, source: Expression, destination: Expression },
    TTest {
        sample:      Option<String>,
        column:      Expression,
        group1:      Expression,
        group2:      Expression,
        destination: Expression,
    },
    Anova {
        column:      Expression,
        group1:      Expression,
        group2:      Expression,
        destination: Expression,
    },
    MedianTest {
        column:      Expression,
        group1:      Expression,
        group2:      Expression,
        destination: Expression,
    },

    // ── Regression ────────────────────────────────────────────────────────────
    Regress {
        target:      Expression,
        features:    Vec<Expression>,
        source:      Expression,
        destination: Expression,
        with_openai: bool,
    },

    // ── Arithmetic ────────────────────────────────────────────────────────────
    Arithmetic {
        op:          ArithmeticOp,
        value:       Expression,
        target:      Expression,
        destination: Option<Expression>,
    },

    // ── Collection operations ─────────────────────────────────────────────────
    Filter { source: Expression, condition: FilterCondition, destination: Expression },

    // ── Foreign library calls ─────────────────────────────────────────────────
    Import     { backend: Backend, module: Expression, alias: Expression },
    Install    { backend: Backend, module: Expression },
    Call       { library: Expression, function: Expression, args: Vec<Expression>, destination: Expression },
    CallMethod { object: Expression, method: Expression, args: Vec<Expression>, destination: Expression },

    // ── OpenAI / LLM ──────────────────────────────────────────────────────────
    OpenAI {
        prompt:      Expression,
        model:       Option<Expression>,
        destination: Option<Expression>,
        file_path:   Option<Expression>,
    },

    // ── Control flow ──────────────────────────────────────────────────────────
    If {
        condition: Expression,
        then_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
    ForEach {
        item: String,
        list: Expression,
        body: Vec<Statement>,
    },
    While {
        condition: Expression,
        body:      Vec<Statement>,
    },
    Break,
    Continue,

    // ── Error handling ────────────────────────────────────────────────────────
    /// `try ... catch error_var ... end`
    TryCatch {
        try_body:   Vec<Statement>,
        error_var:  String,
        catch_body: Vec<Statement>,
    },

    // ── User-defined functions ────────────────────────────────────────────────
    /// `define the function NAME with inputs A, B, C as ... end`
    FunctionDef {
        name:   String,
        params: Vec<String>,
        body:   Vec<Statement>,
    },
    /// `call NAME with arg1, arg2 as result`
    FunctionCall {
        name:        String,
        args:        Vec<Expression>,
        destination: Option<Expression>,
    },
    Return { value: Option<Expression> },

    // ── Parallelism & concurrency ─────────────────────────────────────────────
    /// `parallel map each ITEM in LIST using N threads do ... end and call it DEST`
    ParallelMap {
        item:        String,
        list:        Expression,
        body:        Vec<Statement>,
        threads:     Option<Expression>,
        destination: Expression,
    },
    /// `parallel for each ITEM in LIST using N threads do ... end`
    ParallelFor {
        item:    String,
        list:    Expression,
        body:    Vec<Statement>,
        threads: Option<Expression>,
    },
    /// `create channel NAME with capacity N`
    CreateChannel {
        name:     String,
        capacity: Option<Expression>,
    },
    /// `send VALUE to channel NAME`
    SendToChannel {
        channel: Expression,
        value:   Expression,
    },
    /// `receive from channel NAME and call it DEST`
    RecvFromChannel {
        channel:     Expression,
        destination: Expression,
    },
    /// `spawn do ... end`  or  `spawn do ... end and join as HANDLE`
    Spawn {
        body:     Vec<Statement>,
        join_as:  Option<String>,
    },

    // ── Memory management ─────────────────────────────────────────────────────
    /// `set memory limit to 512 megabytes`
    SetMemoryLimit { megabytes: Expression },
    /// `print memory usage`
    PrintMemoryUsage,
    /// `with memory scope do ... end`
    MemoryScope { body: Vec<Statement> },

    // ── Database ──────────────────────────────────────────────────────────────
    /// `connect to postgres "url" as db`
    DbConnect {
        db_type:     Expression,   // "postgres" | "mysql" | "sqlite"
        url:         Expression,   // connection string or env var reference
        alias:       String,
    },
    /// `query "SELECT ..." in db with parameters [x, y] as results`
    DbQuery {
        connection:  Expression,
        sql:         Expression,
        params:      Vec<Expression>,
        destination: Expression,
    },
    /// `run "UPDATE ..." in db with parameters [x]`
    DbExecute {
        connection: Expression,
        sql:        Expression,
        params:     Vec<Expression>,
    },
    /// `insert data into db table "my_table"`
    DbInsert {
        connection: Expression,
        table:      Expression,
        data:       Expression,
    },
    /// `close db`
    DbClose { connection: Expression },
}
