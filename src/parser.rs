// src/parser.rs

use crate::ast::{Expression, Statement};
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => return Err(e),
            }
        }
        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current_token() {
            Some(Token::Load) => self.parse_load_statement(),
            Some(Token::Print) => self.parse_print_statement(),
            _ => Err(format!("Unexpected token: {:?}", self.current_token())),
        }
    }

    fn parse_load_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // Consume 'load'
        let source = self.parse_expression()?;

        if self.consume(Token::From).is_err() && self.consume(Token::As).is_ok() {
            // Simple form: load "file.txt" as data
            let alias = self.parse_expression()?;
            return Ok(Statement::Load { source, alias });
        }
        // Could expand here for `load data from source`
        Err("Invalid load statement syntax".to_string())
    }
    
    fn parse_print_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // Consume 'print'
        let expr = self.parse_expression()?;
        Ok(Statement::Print(expr))
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        match self.current_token() {
            Some(Token::String(value)) => {
                let val = value.clone();
                self.advance();
                Ok(Expression::StringLiteral(val))
            }
            Some(Token::Identifier(name)) => {
                let val = name.clone();
                self.advance();
                Ok(Expression::Identifier(val))
            }
            _ => Err("Expected an expression (Identifier or String)".to_string()),
        }
    }
    
    // Helper methods
    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len() || self.current_token() == Some(&Token::Eof)
    }

    fn consume(&mut self, expected: Token) -> Result<(), String> {
        if let Some(token) = self.current_token() {
            if std::mem::discriminant(token) == std::mem::discriminant(&expected) {
                self.advance();
                return Ok(());
            }
        }
        Err(format!("Expected {:?}, found {:?}", expected, self.current_token()))
    }
}