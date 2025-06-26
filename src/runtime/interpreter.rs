use std::collections::HashSet;
use std::fs;

// FFI Imports
use pyo3::prelude::*;
// FIX: Import the prelude with an alias to avoid name collisions.
use extendr_api::prelude as r_prelude;
// FIX: We are no longer using the jni crate directly in this simplified version.

// AST and Runtime Imports
use crate::ast::{ArithmeticOp, Backend, Expression, FilterCondition, Statement};
use crate::runtime::{Environment, TaleaValue};
use crate::lexer::Token;

pub struct Interpreter {
    environment: Environment,
    active_backends: HashSet<Backend>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
            active_backends: HashSet::new(),
        }
    }

    // FIX: Explicitly use `std::result::Result` to avoid name collision with extendr's Result type.
    // This fix is applied to all function signatures in this file.
    pub fn execute(&mut self, statements: Vec<Statement>) -> std::result::Result<(), String> {
        // FIX: Iterate by reference to fix the mismatched types error.
        for statement in &statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, statement: &Statement) -> std::result::Result<(), String> {
        match statement {
            Statement::Use(backend) => self.execute_use_statement(backend),
            Statement::Summarize { source, destination } => self.execute_summarize_statement(source, destination),
            Statement::Tag { source, method, destination } => self.execute_tag_statement(source, method, destination),
            Statement::Define { name, value } => self.execute_define_statement(name, value),
            Statement::Arithmetic { op, value, target, destination } => self.execute_arithmetic_statement(op, value, target, destination),
            Statement::Load { source, alias } => self.execute_load_statement(source, alias),
            Statement::Save { source, destination } => self.execute_save_statement(source, destination),
            Statement::Print(expression) => self.execute_print_statement(expression),
            Statement::Tokenize { source, destination } => self.execute_tokenize_statement(source, destination),
            Statement::Count { unit, source, destination } => self.execute_count_statement(unit, source, destination),
            Statement::Lemmatize { source, destination } => self.execute_lemmatize_statement(source, destination),
            Statement::Filter { source, condition, destination } => self.execute_filter_statement(source, condition, destination),
        }
    }

    fn execute_use_statement(&mut self, backend: &Backend) -> std::result::Result<(), String> {
        self.active_backends.insert(backend.clone());
        println!("[Interpreter: {:?} backend enabled.]", backend);
        // The R engine is now initialized automatically by the `test!` macro when it's needed.
        // We no longer need any special code here.
        Ok(())

    }

    fn execute_summarize_statement(&mut self, source: &Expression, destination: &Expression) -> std::result::Result<(), String> {
        if !self.active_backends.contains(&Backend::R) { return Err("The 'summarize' command requires the R backend. Run 'use r' first.".to_string()); }
        let source_list = if let TaleaValue::List(l) = self.evaluate_expression(source)? { l } else { return Err("Summarize can only be applied to a list of numbers.".to_string()); };
        let dest_name = self.get_identifier_name(destination)?;
        let numbers: Vec<f64> = source_list.iter().filter_map(|val| if let TaleaValue::Number(n) = val { Some(*n as f64) } else { None }).collect();
        println!("[Interpreter: Calling R to summarize data...]");
        
        // FIX: Use the aliased prelude and handle the R result type correctly.
        r_prelude::test! {
                // Construct the R command as a string
            let r_command = format!("summary(c({}))", numbers.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(","));
            // Evaluate the string and get the result
            let summary_result = r_prelude::eval_string(&r_command).map_err(|e| e.to_string())?;
            let summary_output = format!("{:?}", summary_result);
            self.environment.define(dest_name, TaleaValue::String(summary_output));
        }
        Ok(())
    }

    fn execute_tag_statement(&mut self, source: &Expression, method: &Expression, destination: &Expression) -> std::result::Result<(), String> {
        // FIX: Simplified to only use Python for now.
        if self.active_backends.contains(&Backend::Python) {
            self.execute_tag_python(source, method, destination)
        } else {
            Err("The 'tag' command currently requires the Python backend. Please run 'use python' first.".to_string())
        }
    }

    fn execute_tag_python(&mut self, source: &Expression, method: &Expression, destination: &Expression) -> std::result::Result<(), String> {
        let text = self.get_string_value(source)?;
        let method_token = if let Expression::Unit(t) = method { t } else { return Err("Invalid method for tag".to_string()); };
        let dest_name = self.get_identifier_name(destination)?;
        println!("[Interpreter: Calling Python/spaCy for {:?} tagging...]", method_token);
        Python::with_gil(|py| -> PyResult<()> {
            let spacy = PyModule::import_bound(py, "spacy")?;
            let nlp = spacy.call_method1("load", ("en_core_web_md",))?;
            let doc = nlp.call1((&text,))?;
            let result_list = match method_token {
                Token::POS => {
                    doc.iter()?.map(|token_result| {
                        let token = token_result?;
                        let text = token.getattr("text")?.extract::<String>()?;
                        let pos = token.getattr("pos_")?.extract::<String>()?;
                        Ok(TaleaValue::Tuple(vec![TaleaValue::String(text), TaleaValue::String(pos)]))
                    }).collect::<PyResult<Vec<TaleaValue>>>()?
                },
                Token::NER => {
                    doc.getattr("ents")?.iter()?.map(|ent_result| {
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
    
    // ... other execute functions ...
    fn execute_lemmatize_statement(&mut self, source: &Expression, destination: &Expression) -> std::result::Result<(), String> {
        if !self.active_backends.contains(&Backend::Python) { return Err("Lemmatize requires 'use python' first.".to_string()); }
        let text = self.get_string_value(source)?;
        let dest_name = self.get_identifier_name(destination)?;
        println!("[Interpreter: Calling Python/spaCy for lemmatization...]");
        Python::with_gil(|py| -> PyResult<()> {
            let spacy = PyModule::import_bound(py, "spacy")?;
            let nlp = spacy.call_method1("load", ("en_core_web_md",))?;
            let doc = nlp.call1((&text,))?;
            let lemmas = doc.iter()?.map(|token_result| {
                let token = token_result?;
                let lemma = token.getattr("lemma_")?.extract::<String>()?;
                Ok(TaleaValue::String(lemma))
            }).collect::<PyResult<Vec<TaleaValue>>>()?;
            self.environment.define(dest_name, TaleaValue::List(lemmas));
            Ok(())
        }).map_err(|e| format!("Python Error: {}", e))?;
        println!("[Interpreter: Lemmatization complete.]");
        Ok(())
    }

    fn execute_filter_statement(&mut self, source: &Expression, condition: &FilterCondition, destination: &Expression) -> std::result::Result<(), String> {
        let source_list = if let TaleaValue::List(l) = self.evaluate_expression(source)? { l } else { return Err("Filter can only be applied to a list.".to_string()); };
        let dest_name = self.get_identifier_name(destination)?;
        let mut filtered_list = Vec::new();
        for item in source_list {
            if let Some(s_item) = item.as_string() {
                let passes = match condition {
                    FilterCondition::Containing(expr) => self.evaluate_expression(expr)?.as_string().map_or(false, |p| s_item.contains(&p)),
                    FilterCondition::StartingWith(expr) => self.evaluate_expression(expr)?.as_string().map_or(false, |p| s_item.starts_with(&p)),
                    FilterCondition::EndingWith(expr) => self.evaluate_expression(expr)?.as_string().map_or(false, |p| s_item.ends_with(&p)),
                };
                if passes { filtered_list.push(item); }
            }
        }
        println!("[Interpreter: Filtered list contains {} items.]", filtered_list.len());
        self.environment.define(dest_name, TaleaValue::List(filtered_list));
        Ok(())
    }

    fn execute_save_statement(&mut self, source: &Expression, destination: &Expression) -> std::result::Result<(), String> { let source_val = self.evaluate_expression(source)?; let file_path = self.get_string_value(destination)?; let content_to_save = source_val.to_string(); fs::write(&file_path, content_to_save).map_err(|e| format!("Failed to write to file '{}': {}", file_path, e))?; println!("[Interpreter: Successfully saved content to '{}']", file_path); Ok(()) }
    fn execute_arithmetic_statement(&mut self, op: &ArithmeticOp, value: &Expression, target: &Expression, destination: &Option<Expression>) -> std::result::Result<(), String> { let target_name = self.get_identifier_name(target)?; let current_val = self.environment.get(&target_name).ok_or_else(|| format!("Variable '{}' not found.", target_name))?; let current_num = if let TaleaValue::Number(n) = current_val { n } else { return Err(format!("Cannot perform arithmetic on '{}'. Not a number.", target_name)); }; let operand_val = self.evaluate_expression(value)?; let operand_num = if let TaleaValue::Number(n) = operand_val { n } else { return Err("Arithmetic operations require a number.".to_string()); }; let result = match op { ArithmeticOp::Add => current_num + operand_num, ArithmeticOp::Subtract => current_num - operand_num, ArithmeticOp::Multiply => current_num * operand_num, ArithmeticOp::Divide => { if operand_num == 0 { return Err("Division by zero.".to_string()); } current_num / operand_num } }; let final_dest_name = if let Some(dest_expr) = destination { self.get_identifier_name(dest_expr)? } else { target_name.clone() }; self.environment.define(final_dest_name, TaleaValue::Number(result)); Ok(()) }
    fn execute_define_statement(&mut self, name: &Expression, value: &Expression) -> std::result::Result<(), String> { let var_name = self.get_identifier_name(name)?; let value = self.evaluate_expression(value)?; self.environment.define(var_name, value); Ok(()) }
    fn execute_load_statement(&mut self, source: &Expression, alias: &Expression) -> std::result::Result<(), String> { let var_name = self.get_identifier_name(alias)?; let file_path = self.get_string_value(source)?; let file_content = fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file '{}': {}", file_path, e))?; println!("[Interpreter: Successfully read {} bytes from '{}']", file_content.len(), file_path); self.environment.define(var_name, TaleaValue::String(file_content)); Ok(()) }
    fn execute_tokenize_statement(&mut self, source: &Expression, destination: &Expression) -> std::result::Result<(), String> { let source_val = self.evaluate_expression(source)?; let dest_name = self.get_identifier_name(destination)?; if let TaleaValue::String(s) = source_val { let tokens: Vec<TaleaValue> = s.split_whitespace().map(|word| TaleaValue::String(word.to_string())).collect(); println!("[Interpreter: Tokenized text into {} tokens.]", tokens.len()); self.environment.define(dest_name, TaleaValue::List(tokens)); Ok(()) } else { Err("The 'tokenize' command can only be used on a String value.".to_string()) } }
    fn execute_count_statement(&mut self, unit: &Expression, source: &Expression, destination: &Expression) -> std::result::Result<(), String> {
            let unit_token = if let Expression::Unit(t) = unit { t } else { return Err("Invalid unit for count".to_string()); };
            let source_val = self.evaluate_expression(source)?;
            let dest_name = self.get_identifier_name(destination)?;

            match (unit_token, source_val) {
                // CASE 1: Count items in a list (e.g., "count words in word_list")
                (Token::Words | Token::Tokens, TaleaValue::List(l)) => {
                    let count = l.len() as i64;
                    println!("[Interpreter: Counted {} items]", count);
                    self.environment.define(dest_name, TaleaValue::Number(count));
                },
                // CASE 2: Get character/line count from a single string
                (Token::Characters, TaleaValue::String(s)) => {
                    let count = s.len() as i64;
                    println!("[Interpreter: Counted {} characters]", count);
                    self.environment.define(dest_name, TaleaValue::Number(count));
                },
                (Token::Lines, TaleaValue::String(s)) => {
                    let count = s.lines().count() as i64;
                    println!("[Interpreter: Counted {} lines]", count);
                    self.environment.define(dest_name, TaleaValue::Number(count));
                },
                
                // CASE 3: Get character count for *each item* in a list
                (Token::Characters, TaleaValue::List(l)) => {
                    let lengths: Vec<TaleaValue> = l.iter().map(|item| {
                        let len = if let TaleaValue::String(s) = item {
                            s.len() as i64
                        } else {
                            0 // Or handle error appropriately
                        };
                        TaleaValue::Number(len)
                    }).collect();
                    println!("[Interpreter: Generated list of {} character counts]", lengths.len());
                    self.environment.define(dest_name, TaleaValue::List(lengths));
                },
                _ => return Err(format!("Cannot count {:?} in the provided variable type.", unit_token)),
            }
            Ok(())
        }
    fn execute_print_statement(&mut self, expression: &Expression) -> std::result::Result<(), String> { let value = self.evaluate_expression(expression)?; println!("{}", value.to_string()); Ok(()) }
    fn evaluate_expression(&mut self, expression: &Expression) -> std::result::Result<TaleaValue, String> { match expression { Expression::StringLiteral(s) => Ok(TaleaValue::String(s.clone())), Expression::Number(n) => Ok(TaleaValue::Number(*n)), Expression::Identifier(name) => self.environment.get(name).ok_or_else(|| format!("Variable '{}' not found.", name)), Expression::Unit(token) => Ok(TaleaValue::Unit(token.clone())), } }
    fn get_identifier_name(&self, expression: &Expression) -> std::result::Result<String, String> { if let Expression::Identifier(name) = expression { Ok(name.clone()) } else { Err("Expected an identifier".to_string()) } }
    fn get_string_value(&mut self, expression: &Expression) -> std::result::Result<String, String> { if let TaleaValue::String(s) = self.evaluate_expression(expression)? { Ok(s) } else { Err("Expected a string value".to_string()) } }
}

impl TaleaValue {
    fn as_string(&self) -> Option<String> { if let TaleaValue::String(s) = self { Some(s.clone()) } else { None } }
}
impl ToString for TaleaValue {
    fn to_string(&self) -> String {
        match self {
            TaleaValue::String(s) => s.clone(),
            TaleaValue::Number(n) => n.to_string(),
            TaleaValue::List(l) => {
                let items: Vec<String> = l.iter().map(|val| val.to_string_for_list()).collect();
                format!("[List with {} items]:\n[{}]", l.len(), items.join(", "))
            },
            TaleaValue::Tuple(t) => {
                let items: Vec<String> = t.iter().map(|val| val.to_string_for_list()).collect();
                format!("({})", items.join(", "))
            },
            TaleaValue::Unit(t) => format!("Unit: {:?}", t),
            TaleaValue::Null => "null".to_string(),
        }
    }
}
trait ToStringForList { fn to_string_for_list(&self) -> String; }
impl ToStringForList for TaleaValue {
    fn to_string_for_list(&self) -> String {
        match self {
            TaleaValue::String(s) => format!("'{}'", s),
            _ => self.to_string(),
        }
    }
}