// src/ast.rs
use crate::lexer::Token;

// ... Expression, ArithmeticOp, FilterCondition enums are the same ...
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Identifier(String),
    StringLiteral(String),
    Number(i64),
    Unit(Token),
    ListLiteral(Vec<Expression>),
    Index(Box<Expression>, Box<Expression>),
    Comparison(Box<Expression>, ComparisonOp, Box<Expression>),
}
#[derive(Debug, PartialEq)]
pub enum ArithmeticOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ComparisonOp {
    Equals,
    GreaterThan,
    LessThan,
}
#[derive(Debug, PartialEq)]
pub enum FilterCondition {
    Containing(Expression),
    StartingWith(Expression),
    EndingWith(Expression),
}

// New enum to represent the language backends
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Backend {
    Python,
    R,
    Java,
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Use(Backend), // New: `use python`
    Help,
    Load {
        source: Expression,
        alias: Expression,
    },
    LoadCsv {
        path: Expression,
        alias: Expression,
    },
    Save {
        source: Expression,
        destination: Expression,
    },
    Print(Expression),
    Define {
        name: Expression,
        value: Expression,
    },
    Tokenize {
        source: Expression,
        destination: Expression,
    },
    Doc2Bow {
        source: Expression,
        destination: Expression,
    },
    Split {
        unit: Expression,
        source: Expression,
        destination: Expression,
    },
    Join {
        unit: Expression,
        source: Expression,
        separator: Expression,
        destination: Expression,
    },
    Replace {
        pattern: Expression,
        replacement: Expression,
        source: Expression,
        destination: Expression,
    },
    Clean {
        source: Expression,
        destination: Expression,
    },
    Lowercase {
        source: Expression,
        destination: Expression,
    },
    Uppercase {
        source: Expression,
        destination: Expression,
    },
    Stem {
        source: Expression,
        destination: Expression,
    },
    Normalize {
        source: Expression,
        destination: Expression,
    },
    Count {
        unit: Expression,
        source: Expression,
        destination: Expression,
    },
    Concordance {
        word: Expression,
        source: Expression,
        destination: Expression,
    },
    Tag {
        source: Expression,
        method: Expression,
        destination: Expression,
    },
    Translate {
        source: Expression,
        language: Expression,
        destination: Expression,
    },
    Sentiment {
        source: Expression,
        destination: Expression,
    },
    Mean {
        column: Expression,
        source: Expression,
        destination: Expression,
    },
    Median {
        column: Expression,
        source: Expression,
        destination: Expression,
    },
    Mode {
        column: Expression,
        source: Expression,
        destination: Expression,
    },
    Skewness {
        column: Expression,
        source: Expression,
        destination: Expression,
    },
    Kurtosis {
        column: Expression,
        source: Expression,
        destination: Expression,
    },
    Arithmetic {
        op: ArithmeticOp,
        value: Expression,
        target: Expression,
        destination: Option<Expression>,
    },
    Lemmatize {
        source: Expression,
        destination: Expression,
    },
    Filter {
        source: Expression,
        condition: FilterCondition,
        destination: Expression,
    },
    Summarize {
        source: Expression,
        destination: Expression,
    }, // New
    WordCloud {
        source: Expression,
        destination: Expression,
    },
    WordAssociation {
        source: Expression,
        destination: Expression,
    },
    SemanticSimilarity {
        text1: Expression,
        text2: Expression,
        destination: Expression,
    },
    SemanticSearch {
        query: Expression,
        source: Expression,
        destination: Expression,
    },
    KeywordSearch {
        query: Expression,
        source: Expression,
        destination: Expression,
    },
    Import {
        backend: Backend,
        module: Expression,
        alias: Expression,
    },
    Install {
        backend: Backend,
        module: Expression,
    },
    Call {
        library: Expression,
        function: Expression,
        args: Vec<Expression>,
        destination: Expression,
    },
    OpenAI {
        prompt: Expression,
        model: Option<Expression>,
        destination: Option<Expression>,
        file_path: Option<Expression>,
    },
    SetEnv {
        name: String,
        value: Expression,
    },
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
}
