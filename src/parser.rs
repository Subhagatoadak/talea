// src/parser.rs
use crate::ast::{ArithmeticOp, Expression, FilterCondition, Statement};
use crate::lexer::Token;

pub struct Parser { tokens: Vec<Token>, position: usize }
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { Parser { tokens, position: 0 } }
    pub fn parse(&mut self) -> Result<Vec<Statement>, String> { let mut stmts = Vec::new(); while !self.is_at_end() { stmts.push(self.parse_statement()?); } Ok(stmts) }
    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current_token().cloned() {
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
    fn parse_lemmatize_statement(&mut self) -> Result<Statement, String> { self.advance(); let source = self.parse_expression()?; self.consume(Token::As)?; let destination = self.parse_expression()?; Ok(Statement::Lemmatize { source, destination }) }
    fn parse_filter_statement(&mut self) -> Result<Statement, String> {
        self.advance(); let source = self.parse_expression()?;
        let condition = match self.current_token() {
            Some(Token::Containing) => { self.advance(); FilterCondition::Containing(self.parse_expression()?) }
            Some(Token::StartingWith) => { self.advance(); FilterCondition::StartingWith(self.parse_expression()?) }
            Some(Token::EndingWith) => { self.advance(); FilterCondition::EndingWith(self.parse_expression()?) }
            _ => return Err("Expected a filter condition like 'containing'".to_string())
        };
        self.consume(Token::As)?; let destination = self.parse_expression()?;
        Ok(Statement::Filter { source, condition, destination })
    }
    fn parse_arithmetic_statement(&mut self, op: ArithmeticOp) -> Result<Statement, String> { self.advance(); let (target, value) = match op { ArithmeticOp::Add | ArithmeticOp::Subtract => { let val = self.parse_expression()?; if self.current_token() == Some(&Token::To) || self.current_token() == Some(&Token::From) { self.advance(); } else { return Err("Expected 'to' or 'from'".to_string()); } let tar = self.parse_expression()?; (tar, val) }, ArithmeticOp::Multiply | ArithmeticOp::Divide => { let tar = self.parse_expression()?; self.consume(Token::By)?; let val = self.parse_expression()?; (tar, val) } }; let destination = if self.current_token() == Some(&Token::As) { self.advance(); Some(self.parse_expression()?) } else { None }; Ok(Statement::Arithmetic { op, value, target, destination }) }
    fn parse_define_statement(&mut self) -> Result<Statement, String> { self.advance(); let name = self.parse_expression()?; self.consume(Token::As)?; let value = self.parse_expression()?; Ok(Statement::Define { name, value }) }
    fn parse_count_statement(&mut self) -> Result<Statement, String> { self.advance(); let unit = self.parse_unit_expression()?; self.consume(Token::In)?; let source = self.parse_expression()?; self.consume(Token::As)?; let destination = self.parse_expression()?; Ok(Statement::Count { unit, source, destination }) }
    fn parse_tag_statement(&mut self) -> Result<Statement, String> { self.advance(); let source = self.parse_expression()?; self.consume(Token::With)?; let method = self.parse_unit_expression()?; self.consume(Token::As)?; let destination = self.parse_expression()?; Ok(Statement::Tag { source, method, destination }) }
    fn parse_load_statement(&mut self) -> Result<Statement, String> { self.advance(); let source = self.parse_expression()?; self.consume(Token::As)?; let alias = self.parse_expression()?; Ok(Statement::Load { source, alias }) }
    fn parse_save_statement(&mut self) -> Result<Statement, String> { self.advance(); let source = self.parse_expression()?; self.consume(Token::To)?; let destination = self.parse_expression()?; Ok(Statement::Save { source, destination }) }
    fn parse_print_statement(&mut self) -> Result<Statement, String> { self.advance(); let expr = self.parse_expression()?; Ok(Statement::Print(expr)) }
    fn parse_tokenize_statement(&mut self) -> Result<Statement, String> { self.advance(); let source = self.parse_expression()?; self.consume(Token::As)?; let destination = self.parse_expression()?; Ok(Statement::Tokenize { source, destination }) }
    fn parse_expression(&mut self) -> Result<Expression, String> { if self.is_unit_token() { return self.parse_unit_expression(); } match self.current_token().cloned() { Some(Token::String(v)) => { self.advance(); Ok(Expression::StringLiteral(v)) } Some(Token::Identifier(n)) => { self.advance(); Ok(Expression::Identifier(n)) } Some(Token::Number(v)) => { self.advance(); Ok(Expression::Number(v)) } _ => Err(format!("Expected an expression, but found {:?}", self.current_token())), } }
    fn parse_unit_expression(&mut self) -> Result<Expression, String> { match self.current_token() { Some(tok) if self.is_unit_token() => { let unit_token = tok.clone(); self.advance(); Ok(Expression::Unit(unit_token)) }, _ => Err(format!("Expected a unit (like words, ner, etc), but found {:?}", self.current_token())), } }
    fn is_unit_token(&self) -> bool { matches!(self.current_token(), Some(Token::Words) | Some(Token::Sentences) | Some(Token::Lines) | Some(Token::Characters) | Some(Token::Tokens) | Some(Token::POS) | Some(Token::NER)) }
    fn current_token(&self) -> Option<&Token> { self.tokens.get(self.position) }
    fn advance(&mut self) { if !self.is_at_end() { self.position += 1; } }
    fn is_at_end(&self) -> bool { self.position >= self.tokens.len() || self.current_token() == Some(&Token::Eof) }
    fn consume(&mut self, expected: Token) -> Result<(), String> { if let Some(t) = self.current_token() { if std::mem::discriminant(t) == std::mem::discriminant(&expected) { self.advance(); return Ok(()); } } Err(format!("Expected {:?}, found {:?}", expected, self.current_token())) }
}