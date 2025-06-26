// src/ast.rs
use crate::lexer::Token;

// ... Expression, ArithmeticOp, FilterCondition enums are the same ...
#[derive(Debug, PartialEq, Clone)]
pub enum Expression { Identifier(String), StringLiteral(String), Number(i64), Unit(Token) }
#[derive(Debug, PartialEq)]
pub enum ArithmeticOp { Add, Subtract, Multiply, Divide }
#[derive(Debug, PartialEq)]
pub enum FilterCondition { Containing(Expression), StartingWith(Expression), EndingWith(Expression) }

// New enum to represent the language backends
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Backend { Python, R, Java }

#[derive(Debug, PartialEq)]
pub enum Statement {
    Use(Backend), // New: `use python`
    Load { source: Expression, alias: Expression },
    Save { source: Expression, destination: Expression },
    Print(Expression),
    Define { name: Expression, value: Expression },
    Tokenize { source: Expression, destination: Expression },
    Count { unit: Expression, source: Expression, destination: Expression },
    Tag { source: Expression, method: Expression, destination: Expression },
    Arithmetic { op: ArithmeticOp, value: Expression, target: Expression, destination: Option<Expression> },
    Lemmatize { source: Expression, destination: Expression },
    Filter { source: Expression, condition: FilterCondition, destination: Expression },
    Summarize { source: Expression, destination: Expression }, // New
}