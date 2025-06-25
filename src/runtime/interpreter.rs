// src/runtime/interpreter.rs

// Import the Rust standard library for File System operations
use std::fs;

use crate::ast::{Expression, Statement};
use crate::runtime::{Environment, TaleaValue};

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn execute(&mut self, statements: Vec<Statement>) -> Result<(), String> {
        for statement in statements {
            self.execute_statement(&statement)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Load { source, alias } => self.execute_load_statement(source, alias),
            Statement::Print(expression) => self.execute_print_statement(expression),
        }
    }

    // === Statement Implementations ===

    fn execute_load_statement(
        &mut self,
        source: &Expression,
        alias: &Expression,
    ) -> Result<(), String> {
        let var_name = match alias {
            Expression::Identifier(name) => name.clone(),
            _ => return Err("Expected an identifier for the alias in 'load' statement".to_string()),
        };

        let file_path = match source {
            Expression::StringLiteral(path) => path.clone(),
            _ => return Err("Expected a string literal for the source in 'load' statement".to_string()),
        };

        // --- UPDATED CODE ---
        // 3. Perform the side effect: read the file from the disk.
        // The `?` operator will automatically convert an error into the Err part of our Result,
        // stopping execution and propagating the error message.
        let file_content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read file '{}': {}", file_path, e))?;

        println!("[Interpreter: Successfully read {} bytes from '{}']", file_content.len(), file_path);

        // 4. Store the real content in the environment
        self.environment
            .define(var_name, TaleaValue::String(file_content));
        
        Ok(())
    }

    fn execute_print_statement(&mut self, expression: &Expression) -> Result<(), String> {
        let value = self.evaluate_expression(expression)?;
        match value {
            TaleaValue::String(s) => println!("{}", s),
            TaleaValue::Null => println!("null"),
        }
        Ok(())
    }
    
    // === Expression Evaluation ===
    
    fn evaluate_expression(&self, expression: &Expression) -> Result<TaleaValue, String> {
        match expression {
            Expression::StringLiteral(s) => Ok(TaleaValue::String(s.clone())),
            Expression::Identifier(name) => {
                match self.environment.get(name) {
                    Some(value) => Ok(value),
                    None => Err(format!("Variable '{}' not found.", name)),
                }
            }
        }
    }
}