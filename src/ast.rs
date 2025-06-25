// src/ast.rs

#[derive(Debug, PartialEq)]
pub enum Statement {
    Load {
        source: Expression,
        alias: Expression,
    },
    Print(Expression),
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Identifier(String),
    StringLiteral(String),
}