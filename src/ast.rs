// src/ast.rs
use crate::lexer::Token;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression { Identifier(String), StringLiteral(String), Number(i64), Unit(Token) }

#[derive(Debug, PartialEq)]
pub enum ArithmeticOp { Add, Subtract, Multiply, Divide }

#[derive(Debug, PartialEq)]
pub enum FilterCondition {
    Containing(Expression),
    StartingWith(Expression),
    EndingWith(Expression),
}

#[derive(Debug, PartialEq)]
pub enum Statement {
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
}