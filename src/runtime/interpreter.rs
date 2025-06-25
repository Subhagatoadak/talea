// src/runtime/interpreter.rs

use std::fs;
use pyo3::prelude::*;
use crate::ast::{ArithmeticOp, Expression, FilterCondition, Statement};
use crate::runtime::{Environment, TaleaValue};
use crate::lexer::Token;

pub struct Interpreter { environment: Environment }
impl Interpreter {
    pub fn new() -> Self { Interpreter { environment: Environment::new() } }
    pub fn execute(&mut self, statements: Vec<Statement>) -> Result<(), String> {
        for statement in statements { self.execute_statement(&statement)?; }
        Ok(())
    }

    fn execute_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Define { name, value } => self.execute_define_statement(name, value),
            Statement::Arithmetic { op, value, target, destination } => self.execute_arithmetic_statement(op, value, target, destination),
            Statement::Load { source, alias } => self.execute_load_statement(source, alias),
            Statement::Save { source, destination } => self.execute_save_statement(source, destination),
            Statement::Print(expression) => self.execute_print_statement(expression),
            Statement::Tokenize { source, destination } => self.execute_tokenize_statement(source, destination),
            Statement::Count { unit, source, destination } => self.execute_count_statement(unit, source, destination),
            Statement::Tag { source, method, destination } => self.execute_tag_statement(source, method, destination),
            Statement::Lemmatize { source, destination } => self.execute_lemmatize_statement(source, destination),
            Statement::Filter { source, condition, destination } => self.execute_filter_statement(source, condition, destination),
        }
    }

    fn execute_tag_statement(&mut self, source: &Expression, method: &Expression, destination: &Expression) -> Result<(), String> {
        let text = self.get_string_value(source)?;
        let method_token = if let Expression::Unit(t) = method { t } else { return Err("Invalid method for tag".to_string()); };
        let dest_name = self.get_identifier_name(destination)?;

        println!("[Interpreter: Calling Python/spaCy for {:?} tagging...]", method_token);

        Python::with_gil(|py| -> PyResult<()> {
            let spacy = PyModule::import_bound(py, "spacy")?;
            let nlp = spacy.call_method1("load", ("en_core_web_md",))?;
            let doc = nlp.call1((text,))?;
            
            let result_list = match method_token {
                Token::POS => {
                    // FIX: Iterate directly over tokens in the doc to avoid lifetime issues.
                    doc.iter()?.map(|token_result| {
                        let token = token_result?; // Handle the Result for each token
                        let text = token.getattr("text")?.extract::<String>()?;
                        let pos = token.getattr("pos_")?.extract::<String>()?;
                        Ok(TaleaValue::Tuple(vec![TaleaValue::String(text), TaleaValue::String(pos)]))
                    }).collect::<PyResult<Vec<TaleaValue>>>()?
                },
                Token::NER => {
                    doc.getattr("ents")?.iter()?.map(|ent_result| {
                        // FIX: Handle the Result for each entity first.
                        let ent = ent_result?;
                        let text = ent.getattr("text")?.extract::<String>()?;
                        let label = ent.getattr("label_")?.extract::<String>()?;
                        Ok(TaleaValue::Tuple(vec![TaleaValue::String(text), TaleaValue::String(label)]))
                    }).collect::<PyResult<Vec<TaleaValue>>>()?
                },
                _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Unsupported tagging method"))
            };

            self.environment.define(dest_name, TaleaValue::List(result_list));
            Ok(())
        }).map_err(|e| format!("Python Error: {}", e))?;
        
        println!("[Interpreter: Tagging complete.]");
        Ok(())
    }

    fn execute_lemmatize_statement(&mut self, source: &Expression, destination: &Expression) -> Result<(), String> {
        let text = self.get_string_value(source)?;
        let dest_name = self.get_identifier_name(destination)?;
        println!("[Interpreter: Calling Python/spaCy for lemmatization...]");
        Python::with_gil(|py| -> PyResult<()> {
            let spacy = PyModule::import_bound(py, "spacy")?;
            let nlp = spacy.call_method1("load", ("en_core_web_md",))?;
            let doc = nlp.call1((text,))?;

            // FIX: Iterate directly over the document's tokens to avoid the lifetime error.
            let lemmas = doc.iter()?.map(|token_result| {
                let token = token_result?; // Handle the Result
                let lemma = token.getattr("lemma_")?.extract::<String>()?;
                Ok(TaleaValue::String(lemma))
            }).collect::<PyResult<Vec<TaleaValue>>>()?;
            
            self.environment.define(dest_name, TaleaValue::List(lemmas));
            Ok(())
        }).map_err(|e| format!("Python Error: {}", e))?;
        println!("[Interpreter: Lemmatization complete.]");
        Ok(())
    }

    fn execute_filter_statement(&mut self, source: &Expression, condition: &FilterCondition, destination: &Expression) -> Result<(), String> {
        let source_list = if let TaleaValue::List(l) = self.evaluate_expression(source)? { l } else { return Err("Filter can only be applied to a list.".to_string()); };
        let dest_name = self.get_identifier_name(destination)?;
        let filtered_list = source_list.into_iter().filter_map(|item| {
            let item_str = if let TaleaValue::String(s) = &item { s } else { return Some(None); };
            let passes = match condition {
                FilterCondition::Containing(expr) => { if let Ok(TaleaValue::String(s)) = self.evaluate_expression(expr) { item_str.contains(&s) } else { false } },
                FilterCondition::StartingWith(expr) => { if let Ok(TaleaValue::String(s)) = self.evaluate_expression(expr) { item_str.starts_with(&s) } else { false } },
                FilterCondition::EndingWith(expr) => { if let Ok(TaleaValue::String(s)) = self.evaluate_expression(expr) { item_str.ends_with(&s) } else { false } },
            };
            if passes { Some(Some(item)) } else { Some(None) }
        }).flatten().collect::<Vec<_>>();
        println!("[Interpreter: Filtered list contains {} items.]", filtered_list.len());
        self.environment.define(dest_name, TaleaValue::List(filtered_list));
        Ok(())
    }
    
    fn execute_arithmetic_statement(&mut self, op: &ArithmeticOp, value: &Expression, target: &Expression, destination: &Option<Expression>) -> Result<(), String> {
        let target_name = self.get_identifier_name(target)?;
        let current_val = self.environment.get(&target_name).ok_or_else(|| format!("Variable '{}' not found.", target_name))?;
        // FIX: Remove `mut` from current_num as it's not needed.
        let current_num = if let TaleaValue::Number(n) = current_val { n } else { return Err(format!("Cannot perform arithmetic on '{}'. Not a number.", target_name)); };
        let operand_val = self.evaluate_expression(value)?;
        let operand_num = if let TaleaValue::Number(n) = operand_val { n } else { return Err("Arithmetic operations require a number.".to_string()); };
        
        let result = match op {
            ArithmeticOp::Add => current_num + operand_num,
            ArithmeticOp::Subtract => current_num - operand_num,
            ArithmeticOp::Multiply => current_num * operand_num,
            ArithmeticOp::Divide => {
                if operand_num == 0 { return Err("Division by zero.".to_string()); }
                current_num / operand_num
            }
        };
        let final_dest_name = if let Some(dest_expr) = destination { self.get_identifier_name(dest_expr)? } else { target_name };
        self.environment.define(final_dest_name, TaleaValue::Number(result));
        Ok(())
    }
    
    // ... all other helper and execute functions remain unchanged ...
    fn execute_save_statement(&mut self, source: &Expression, destination: &Expression) -> Result<(), String> { let source_val = self.evaluate_expression(source)?; let file_path = self.get_string_value(destination)?; let content_to_save = match source_val { TaleaValue::String(s) => s, TaleaValue::Number(n) => n.to_string(), TaleaValue::List(l) => { l.iter().map(|val| match val { TaleaValue::String(s) => s.clone(), TaleaValue::Number(n) => n.to_string(), TaleaValue::Tuple(t) => format!("{:?}", t), _ => format!("{:?}", val) }).collect::<Vec<String>>().join("\n") }, _ => return Err("Cannot save this data type to a file.".to_string()), }; fs::write(&file_path, content_to_save).map_err(|e| format!("Failed to write to file '{}': {}", file_path, e))?; println!("[Interpreter: Successfully saved content to '{}']", file_path); Ok(()) }
    fn execute_define_statement(&mut self, name: &Expression, value: &Expression) -> Result<(), String> { let var_name = self.get_identifier_name(name)?; let value = self.evaluate_expression(value)?; self.environment.define(var_name, value); Ok(()) }
    fn execute_load_statement(&mut self, source: &Expression, alias: &Expression) -> Result<(), String> { let var_name = self.get_identifier_name(alias)?; let file_path = self.get_string_value(source)?; let file_content = fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file '{}': {}", file_path, e))?; println!("[Interpreter: Successfully read {} bytes from '{}']", file_content.len(), file_path); self.environment.define(var_name, TaleaValue::String(file_content)); Ok(()) }
    fn execute_tokenize_statement(&mut self, source: &Expression, destination: &Expression) -> Result<(), String> { let source_val = self.evaluate_expression(source)?; let dest_name = self.get_identifier_name(destination)?; if let TaleaValue::String(s) = source_val { let tokens: Vec<TaleaValue> = s.split_whitespace().map(|word| TaleaValue::String(word.to_string())).collect(); println!("[Interpreter: Tokenized text into {} tokens.]", tokens.len()); self.environment.define(dest_name, TaleaValue::List(tokens)); Ok(()) } else { Err("The 'tokenize' command can only be used on a String value.".to_string()) } }
    fn execute_count_statement(&mut self, unit: &Expression, source: &Expression, destination: &Expression) -> Result<(), String> { let unit_token = if let Expression::Unit(t) = unit { t } else { return Err("Invalid unit for count".to_string()); }; let source_val = self.evaluate_expression(source)?; let dest_name = self.get_identifier_name(destination)?; let count = match (unit_token, source_val) { (Token::Words | Token::Tokens, TaleaValue::List(l)) => l.len() as i64, (Token::Characters, TaleaValue::String(s)) => s.len() as i64, (Token::Lines, TaleaValue::String(s)) => s.lines().count() as i64, _ => return Err(format!("Cannot count {:?} in the given variable type.", unit_token)), }; println!("[Interpreter: Counted {} {:?}]", count, unit_token); self.environment.define(dest_name, TaleaValue::Number(count)); Ok(()) }
    fn execute_print_statement(&mut self, expression: &Expression) -> Result<(), String> { let value = self.evaluate_expression(expression)?; match value { TaleaValue::String(s) => println!("{}", s), TaleaValue::Number(n) => println!("{}", n), TaleaValue::List(l) => println!("[List with {} items]:\n{:?}", l.len(), l), TaleaValue::Tuple(t) => println!("Tuple: {:?}", t), _ => println!("{:?}", value), }; Ok(()) }
    fn evaluate_expression(&mut self, expression: &Expression) -> Result<TaleaValue, String> { match expression { Expression::StringLiteral(s) => Ok(TaleaValue::String(s.clone())), Expression::Number(n) => Ok(TaleaValue::Number(*n)), Expression::Identifier(name) => self.environment.get(name).ok_or_else(|| format!("Variable '{}' not found.", name)), Expression::Unit(token) => Ok(TaleaValue::Unit(token.clone())), } }
    fn get_identifier_name(&self, expression: &Expression) -> Result<String, String> { if let Expression::Identifier(name) = expression { Ok(name.clone()) } else { Err("Expected an identifier".to_string()) } }
    fn get_string_value(&mut self, expression: &Expression) -> Result<String, String> { if let TaleaValue::String(s) = self.evaluate_expression(expression)? { Ok(s) } else { Err("Expected a string value".to_string()) } }
}