// src/parser.rs

use crate::ast::{ArithmeticOp, Backend, Expression, FilterCondition, Statement};
use crate::lexer::Token;

pub struct Parser { tokens: Vec<Token>, position: usize }
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { Parser { tokens, position: 0 } }
    pub fn parse(&mut self) -> Result<Vec<Statement>, String> { let mut stmts = Vec::new(); while !self.is_at_end() { stmts.push(self.parse_statement()?); } Ok(stmts) }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current_token().cloned() {
            Some(Token::Use) => self.parse_use_statement(),
            Some(Token::Summarize) => self.parse_summarize_statement(),
            Some(Token::Lemmatize) => self.parse_lemmatize_statement(),
            Some(Token::Filter) => self.parse_filter_statement(),
            Some(Token::Load) => self.parse_load_statement(),
            Some(Token::Save) => self.parse_save_statement(),
            Some(Token::Print) => self.parse_print_statement(),
            Some(Token::Tokenize) => self.parse_tokenize_statement(),
            Some(Token::Count) => self.parse_count_statement(),
            Some(Token::Tag) => self.parse_tag_statement(),
            Some(Token::Define) => self.parse_define_statement(),
            Some(Token::Add) => self.parse_arithmetic_statement(ArithmeticOp::Add),
            Some(Token::Subtract) => self.parse_arithmetic_statement(ArithmeticOp::Subtract),
            Some(Token::Multiply) => self.parse_arithmetic_statement(ArithmeticOp::Multiply),
            Some(Token::Divide) => self.parse_arithmetic_statement(ArithmeticOp::Divide),
            Some(Token::Exit) | Some(Token::Quit) => std::process::exit(0),
            Some(t) => Err(format!("Unrecognized or unimplemented command: {:?}", t)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    // Command Parsers with Context-Aware Logic
    fn parse_load_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let alias = self.parse_identifier_expression()?;
        Ok(Statement::Load { source, alias })
    }

    fn parse_tokenize_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Tokenize { source, destination })
    }
    
    fn parse_tag_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let source = self.parse_expression()?;
        self.consume(Token::With)?;
        let method = self.parse_unit_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Tag { source, method, destination })
    }

    fn parse_count_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let unit = self.parse_unit_expression()?;
        self.consume(Token::In)?;
        let source = self.parse_identifier_expression()?;
        self.consume(Token::As)?;
        let destination = self.parse_identifier_expression()?;
        Ok(Statement::Count { unit, source, destination })
    }

    fn parse_define_statement(&mut self) -> Result<Statement, String> {
        self.advance();
        let name = self.parse_identifier_expression()?;
        self.consume(Token::As)?;
        let value = self.parse_expression()?;
        Ok(Statement::Define { name, value })
    }

    fn parse_arithmetic_statement(&mut self, op: ArithmeticOp) -> Result<Statement, String> {
        self.advance();
        let (target, value) = match op {
            ArithmeticOp::Add | ArithmeticOp::Subtract => {
                let val = self.parse_expression()?;
                if self.current_token() == Some(&Token::To) || self.current_token() == Some(&Token::From) { self.advance(); } else { return Err("Expected 'to' or 'from'".to_string()); }
                let tar = self.parse_expression()?;
                (tar, val)
            },
            ArithmeticOp::Multiply | ArithmeticOp::Divide => {
                let tar = self.parse_expression()?;
                self.consume(Token::By)?;
                let val = self.parse_expression()?;
                (tar, val)
            }
        };
        let destination = if self.current_token() == Some(&Token::As) { self.advance(); Some(self.parse_identifier_expression()?) } else { None };
        Ok(Statement::Arithmetic { op, value, target, destination })
    }
    
    // Other existing parsers
    fn parse_use_statement(&mut self) -> Result<Statement, String> { self.advance(); let backend = match self.current_token() { Some(Token::Python) => Backend::Python, Some(Token::R) => Backend::R, _ => return Err("Expected 'python' or 'r' after 'use'".to_string()) }; self.advance(); Ok(Statement::Use(backend)) }
    fn parse_summarize_statement(&mut self) -> Result<Statement, String> { self.advance(); let source = self.parse_expression()?; self.consume(Token::As)?; let destination = self.parse_identifier_expression()?; Ok(Statement::Summarize { source, destination }) }
    fn parse_lemmatize_statement(&mut self) -> Result<Statement, String> { self.advance(); let source = self.parse_expression()?; self.consume(Token::As)?; let destination = self.parse_identifier_expression()?; Ok(Statement::Lemmatize { source, destination }) }
    fn parse_filter_statement(&mut self) -> Result<Statement, String> { self.advance(); let source = self.parse_expression()?; let condition = match self.current_token() { Some(Token::Containing) => { self.advance(); FilterCondition::Containing(self.parse_expression()?) }, Some(Token::StartingWith) => { self.advance(); FilterCondition::StartingWith(self.parse_expression()?) }, Some(Token::EndingWith) => { self.advance(); FilterCondition::EndingWith(self.parse_expression()?) }, _ => return Err("Expected a filter condition".to_string()) }; self.consume(Token::As)?; let destination = self.parse_identifier_expression()?; Ok(Statement::Filter { source, condition, destination }) }
    fn parse_save_statement(&mut self) -> Result<Statement, String> { self.advance(); let source = self.parse_expression()?; self.consume(Token::To)?; let destination = self.parse_expression()?; Ok(Statement::Save { source, destination }) }
    fn parse_print_statement(&mut self) -> Result<Statement, String> { self.advance(); let expr = self.parse_expression()?; Ok(Statement::Print(expr)) }

    // Expression parsing logic
    fn parse_expression(&mut self) -> Result<Expression, String> {
        let current_token = self.current_token().cloned();
        if self.is_unit_token(&current_token) {
            return self.parse_unit_expression();
        }
        match current_token {
            Some(Token::String(v)) => { self.advance(); Ok(Expression::StringLiteral(v)) },
            Some(Token::Identifier(n)) => { self.advance(); Ok(Expression::Identifier(n)) },
            Some(Token::Number(v)) => { self.advance(); Ok(Expression::Number(v)) },
            Some(tok) => self.parse_keyword_as_identifier(&tok),
            _ => Err(format!("Expected an expression, but found {:?}", self.current_token())),
        }
    }
    
    fn parse_identifier_expression(&mut self) -> Result<Expression, String> {
        match self.current_token().cloned() {
            Some(Token::Identifier(name)) => {
                self.advance();
                Ok(Expression::Identifier(name))
            }
            Some(tok) => self.parse_keyword_as_identifier(&tok),
            None => Err("Expected a variable name, but found nothing.".to_string())
        }
    }
    
    fn parse_unit_expression(&mut self) -> Result<Expression, String> {
        match self.current_token() {
            Some(tok) if self.is_unit_token(&Some(tok.clone())) => {
                let unit_token = tok.clone();
                self.advance();
                Ok(Expression::Unit(unit_token))
            },
            _ => Err(format!("Expected a unit (like words, ner, etc), but found {:?}", self.current_token())),
        }
    }

    fn parse_keyword_as_identifier(&mut self, token: &Token) -> Result<Expression, String> {
        let identifier_string = format!("{:?}", token).to_lowercase();
        self.advance();
        Ok(Expression::Identifier(identifier_string))
    }
    
    fn is_unit_token(&self, token: &Option<Token>) -> bool {
        matches!(token, Some(Token::Words) | Some(Token::Sentences) | Some(Token::Lines) | Some(Token::Paragraphs) | Some(Token::Characters) | Some(Token::Tokens) | Some(Token::Types) | Some(Token::Uniques) | Some(Token::Length) | Some(Token::POS) | Some(Token::NER) | Some(Token::Entities))
    }
    
    fn current_token(&self) -> Option<&Token> { self.tokens.get(self.position) }
    fn advance(&mut self) { if !self.is_at_end() { self.position += 1; } }
    fn is_at_end(&self) -> bool { self.position >= self.tokens.len() || self.current_token() == Some(&Token::Eof) }
    fn consume(&mut self, expected: Token) -> Result<(), String> { if let Some(t) = self.current_token() { if std::mem::discriminant(t) == std::mem::discriminant(&expected) { self.advance(); return Ok(()); } } Err(format!("Expected {:?}, found {:?}", expected, self.current_token())) }
}