// src/compiler.rs
//
// AST → Bytecode compiler for Talea.
// Walks a Vec<Statement> produced by the parser and emits opcodes into a Chunk.

use crate::ast::{ArithmeticOp, Backend, ComparisonOp, Expression, FilterCondition, Statement};
use crate::bytecode::{Chunk, Constant, OpCode};

pub struct Compiler {
    chunk: Chunk,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            chunk: Chunk::new(),
        }
    }

    // ─── Public entry point ───────────────────────────────────────────────────

    pub fn compile_statements(&mut self, stmts: Vec<Statement>) -> Result<(), String> {
        for stmt in stmts {
            self.compile_statement(stmt)?;
        }
        Ok(())
    }

    // ─── Expression compilation ───────────────────────────────────────────────

    fn compile_expr(&mut self, expr: Expression) -> Result<(), String> {
        match expr {
            Expression::StringLiteral(s) => {
                let idx = self.chunk.add_constant(Constant::String(s));
                self.chunk.emit(OpCode::LoadConst(idx));
            }
            Expression::Number(n) => {
                let idx = self.chunk.add_constant(Constant::Number(n));
                self.chunk.emit(OpCode::LoadConst(idx));
            }
            Expression::Identifier(name) => {
                self.chunk.emit(OpCode::LoadVar(name));
            }
            Expression::Unit(token) => {
                // Units are rendered as strings
                let s = format!("{:?}", token);
                let idx = self.chunk.add_constant(Constant::String(s));
                self.chunk.emit(OpCode::LoadConst(idx));
            }
            Expression::ListLiteral(items) => {
                let n = items.len();
                for item in items {
                    self.compile_expr(item)?;
                }
                self.chunk.emit(OpCode::BuildList(n));
            }
            Expression::Index(base, index) => {
                self.compile_expr(*base)?;
                self.compile_expr(*index)?;
                self.chunk.emit(OpCode::Index);
            }
            Expression::Comparison(left, op, right) => {
                self.compile_expr(*left)?;
                self.compile_expr(*right)?;
                match op {
                    ComparisonOp::Equals => {
                        self.chunk.emit(OpCode::CmpEq);
                    }
                    ComparisonOp::GreaterThan => {
                        self.chunk.emit(OpCode::CmpGt);
                    }
                    ComparisonOp::LessThan => {
                        self.chunk.emit(OpCode::CmpLt);
                    }
                }
            }
        }
        Ok(())
    }

    // helper: push the string name from an Identifier expression
    fn compile_name(&mut self, expr: Expression) -> Result<(), String> {
        match expr {
            Expression::Identifier(s) => {
                let idx = self.chunk.add_constant(Constant::String(s));
                self.chunk.emit(OpCode::LoadConst(idx));
                Ok(())
            }
            other => self.compile_expr(other),
        }
    }

    // ─── Statement compilation ────────────────────────────────────────────────

    fn compile_statement(&mut self, stmt: Statement) -> Result<(), String> {
        match stmt {
            // ── Variables / I/O ──────────────────────────────────────────────
            Statement::Define { name, value } => {
                self.compile_expr(value)?;
                self.compile_name(name)?;
                self.chunk.emit(OpCode::Define);
            }
            Statement::Print(expr) => {
                self.compile_expr(expr)?;
                self.chunk.emit(OpCode::Print);
            }
            Statement::Expression(expr) => {
                self.compile_expr(expr)?;
                self.chunk.emit(OpCode::Pop);
            }
            Statement::Load { source, alias } => {
                self.compile_expr(source)?;
                self.compile_name(alias)?;
                self.chunk.emit(OpCode::Load);
            }
            Statement::LoadCsv { path, alias } => {
                self.compile_expr(path)?;
                self.compile_name(alias)?;
                self.chunk.emit(OpCode::LoadCsv);
            }
            Statement::Save {
                source,
                destination,
            } => {
                self.compile_expr(source)?;
                self.compile_expr(destination)?;
                self.chunk.emit(OpCode::Save);
            }

            // ── NLP ──────────────────────────────────────────────────────────
            Statement::Tokenize {
                source,
                destination,
            } => {
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Tokenize);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Sentiment {
                source,
                destination,
            } => {
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Sentiment);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Count {
                unit,
                source,
                destination,
            } => {
                self.compile_expr(unit)?;
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Count);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Split {
                unit,
                source,
                destination,
            } => {
                self.compile_expr(unit)?;
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Split);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Join {
                unit,
                source,
                separator,
                destination,
            } => {
                self.compile_expr(unit)?;
                self.compile_expr(source)?;
                self.compile_expr(separator)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Join);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Replace {
                pattern,
                replacement,
                source,
                destination,
            } => {
                self.compile_expr(pattern)?;
                self.compile_expr(replacement)?;
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Replace);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Clean {
                source,
                destination,
            } => {
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Clean);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Lowercase {
                source,
                destination,
            } => {
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Lowercase);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Uppercase {
                source,
                destination,
            } => {
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Uppercase);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Stem {
                source,
                destination,
            } => {
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Stem);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Normalize {
                source,
                destination,
            } => {
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Normalize);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Lemmatize {
                source,
                destination,
            } => {
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Lemmatize);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Tag {
                source,
                method,
                destination,
            } => {
                self.compile_expr(source)?;
                self.compile_expr(method)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Tag);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Concordance {
                word,
                source,
                destination,
            } => {
                self.compile_expr(word)?;
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Concordance);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::WordCloud {
                source,
                destination,
            } => {
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::WordCloud);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::WordAssociation {
                source,
                destination,
            } => {
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Association);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::SemanticSimilarity {
                text1,
                text2,
                destination,
            } => {
                self.compile_expr(text1)?;
                self.compile_expr(text2)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::SemanticSimilarity);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::SemanticSearch {
                query,
                source,
                destination,
            } => {
                self.compile_expr(query)?;
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::SemanticSearch);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::KeywordSearch {
                query,
                source,
                destination,
            } => {
                self.compile_expr(query)?;
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::KeywordSearch);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Translate {
                source,
                language,
                destination,
            } => {
                self.compile_expr(source)?;
                self.compile_expr(language)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Translate);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Summarize {
                source,
                destination,
            } => {
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Summarize);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Doc2Bow {
                source,
                destination,
            } => {
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Doc2Bow);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Filter {
                source,
                condition,
                destination,
            } => {
                self.compile_expr(source)?;
                // encode the filter condition as a string constant on the stack
                match condition {
                    FilterCondition::Containing(expr) => {
                        let idx = self.chunk.add_constant(Constant::String("containing".to_string()));
                        self.chunk.emit(OpCode::LoadConst(idx));
                        self.compile_expr(expr)?;
                    }
                    FilterCondition::StartingWith(expr) => {
                        let idx = self.chunk.add_constant(Constant::String("starting_with".to_string()));
                        self.chunk.emit(OpCode::LoadConst(idx));
                        self.compile_expr(expr)?;
                    }
                    FilterCondition::EndingWith(expr) => {
                        let idx = self.chunk.add_constant(Constant::String("ending_with".to_string()));
                        self.chunk.emit(OpCode::LoadConst(idx));
                        self.compile_expr(expr)?;
                    }
                }
                let dest = self.extract_ident(&destination)?;
                // reuse Concordance slot; VM interprets it as filter
                self.chunk.emit(OpCode::Concordance); // placeholder – VM handles by context
                self.chunk.emit(OpCode::StoreVar(dest));
            }

            // ── Statistics ───────────────────────────────────────────────────
            Statement::Regress {
                target,
                features,
                source,
                destination,
                with_openai,
            } => {
                self.compile_expr(target)?;
                let n = features.len();
                for f in features {
                    self.compile_expr(f)?;
                }
                let idx = self.chunk.add_constant(Constant::Number(n as i64));
                self.chunk.emit(OpCode::LoadConst(idx));
                self.compile_expr(source)?;
                let wo = if with_openai { 1i64 } else { 0i64 };
                let wi = self.chunk.add_constant(Constant::Number(wo));
                self.chunk.emit(OpCode::LoadConst(wi));
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Regress);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::TTest {
                sample: _,
                column,
                group1,
                group2,
                destination,
            } => {
                self.compile_expr(column)?;
                self.compile_expr(group1)?;
                self.compile_expr(group2)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::TTest);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Anova {
                column,
                group1,
                group2,
                destination,
            } => {
                self.compile_expr(column)?;
                self.compile_expr(group1)?;
                self.compile_expr(group2)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Anova);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::MedianTest {
                column,
                group1,
                group2,
                destination,
            } => {
                // treat as Anova variant (different Python call in VM)
                self.compile_expr(column)?;
                self.compile_expr(group1)?;
                self.compile_expr(group2)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Anova); // VM distinguishes via MedianTest opcode – reuse for now
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Shapiro {
                column,
                data,
                destination,
            } => {
                self.compile_expr(column)?;
                self.compile_expr(data)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Shapiro);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Kolmogorov {
                column,
                data,
                destination,
            } => {
                self.compile_expr(column)?;
                self.compile_expr(data)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Kolmogorov);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Mean {
                column,
                source,
                destination,
            } => {
                self.compile_expr(column)?;
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Mean);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Median {
                column,
                source,
                destination,
            } => {
                self.compile_expr(column)?;
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Median);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Mode {
                column,
                source,
                destination,
            } => {
                self.compile_expr(column)?;
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Mode);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Skewness {
                column,
                source,
                destination,
            } => {
                self.compile_expr(column)?;
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Skewness);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Kurtosis {
                column,
                source,
                destination,
            } => {
                self.compile_expr(column)?;
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Kurtosis);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Insight {
                source,
                destination,
            } => {
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Insight);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Compare {
                left,
                right,
                destination,
            } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::CmpEq);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::Average {
                source,
                destination,
            } => {
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Mean);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::CheckNormality {
                source,
                destination,
            } => {
                self.compile_expr(source)?;
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Shapiro);
                self.chunk.emit(OpCode::StoreVar(dest));
            }

            // ── Arithmetic ───────────────────────────────────────────────────
            Statement::Arithmetic {
                op,
                value,
                target,
                destination,
            } => {
                self.compile_expr(target)?;
                self.compile_expr(value)?;
                let arith_op = match op {
                    ArithmeticOp::Add => OpCode::Add,
                    ArithmeticOp::Subtract => OpCode::Sub,
                    ArithmeticOp::Multiply => OpCode::Mul,
                    ArithmeticOp::Divide => OpCode::Div,
                };
                self.chunk.emit(arith_op);
                if let Some(dest_expr) = destination {
                    let dest = self.extract_ident(&dest_expr)?;
                    self.chunk.emit(OpCode::StoreVar(dest));
                } else {
                    self.chunk.emit(OpCode::Pop);
                }
            }

            // ── Backend / env ────────────────────────────────────────────────
            Statement::Use(backend) => {
                let name = backend_name(&backend);
                self.chunk.emit(OpCode::UseBackend(name));
            }
            Statement::SetEnv { name, value } => {
                let ni = self.chunk.add_constant(Constant::String(name));
                self.chunk.emit(OpCode::LoadConst(ni));
                self.compile_expr(value)?;
                self.chunk.emit(OpCode::SetEnv);
            }
            Statement::Import {
                backend,
                module,
                alias,
            } => {
                let bname = backend_name(&backend);
                let bi = self.chunk.add_constant(Constant::String(bname));
                self.chunk.emit(OpCode::LoadConst(bi));
                self.compile_expr(module)?;
                self.compile_expr(alias)?;
                self.chunk.emit(OpCode::Import);
            }
            Statement::Install { backend: _, module } => {
                // Just compile as a no-op print for now
                self.compile_expr(module)?;
                self.chunk.emit(OpCode::Pop);
            }
            Statement::Call {
                library,
                function,
                args,
                destination,
            } => {
                self.compile_expr(library)?;
                self.compile_expr(function)?;
                let n = args.len();
                for a in args {
                    self.compile_expr(a)?;
                }
                let ni = self.chunk.add_constant(Constant::Number(n as i64));
                self.chunk.emit(OpCode::LoadConst(ni));
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::Call);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::CallMethod {
                object,
                method,
                args,
                destination,
            } => {
                self.compile_expr(object)?;
                self.compile_expr(method)?;
                let n = args.len();
                for a in args {
                    self.compile_expr(a)?;
                }
                let ni = self.chunk.add_constant(Constant::Number(n as i64));
                self.chunk.emit(OpCode::LoadConst(ni));
                let dest = self.extract_ident(&destination)?;
                self.chunk.emit(OpCode::CallMethod);
                self.chunk.emit(OpCode::StoreVar(dest));
            }
            Statement::OpenAI {
                prompt,
                model,
                destination,
                file_path,
            } => {
                self.compile_expr(prompt)?;
                // push model (or null)
                match model {
                    Some(m) => self.compile_expr(m)?,
                    None => {
                        let ni = self.chunk.add_constant(Constant::Null);
                        self.chunk.emit(OpCode::LoadConst(ni));
                    }
                }
                match file_path {
                    Some(fp) => self.compile_expr(fp)?,
                    None => {
                        let ni = self.chunk.add_constant(Constant::Null);
                        self.chunk.emit(OpCode::LoadConst(ni));
                    }
                }
                self.chunk.emit(OpCode::OpenAI);
                match destination {
                    Some(dest_expr) => {
                        let dest = self.extract_ident(&dest_expr)?;
                        self.chunk.emit(OpCode::StoreVar(dest));
                    }
                    None => {
                        self.chunk.emit(OpCode::Print);
                    }
                }
            }

            // ── Control flow ─────────────────────────────────────────────────
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                self.compile_expr(condition)?;
                // emit placeholder JumpIfFalse; remember its index for patching
                let jif_idx = self.chunk.emit(OpCode::JumpIfFalse(0));

                // then-body
                self.compile_statements(then_body)?;

                if else_body.is_empty() {
                    // patch JumpIfFalse to after then-body
                    let target = self.chunk.instructions.len();
                    self.chunk.patch(jif_idx, OpCode::JumpIfFalse(target));
                } else {
                    // emit unconditional jump at end of then-body
                    let jmp_idx = self.chunk.emit(OpCode::Jump(0));
                    // patch JumpIfFalse to start of else-body
                    let else_start = self.chunk.instructions.len();
                    self.chunk.patch(jif_idx, OpCode::JumpIfFalse(else_start));

                    self.compile_statements(else_body)?;

                    // patch Jump to after else-body
                    let after = self.chunk.instructions.len();
                    self.chunk.patch(jmp_idx, OpCode::Jump(after));
                }
            }

            Statement::ForEach { item, list, body } => {
                // stack layout: list, index
                self.compile_expr(list)?;
                // push initial index = 0
                let zi = self.chunk.add_constant(Constant::Number(0));
                self.chunk.emit(OpCode::LoadConst(zi));

                // loop header: compare index < len(list)
                let loop_start = self.chunk.instructions.len();
                self.chunk.emit(OpCode::Label(loop_start));

                // emit ForEach-specific sentinel; VM handles the loop logic
                // We store item name as a constant and use a special pattern:
                // stack: [list, index]
                // ForEachNext pops index, checks bounds, pushes element + new index
                // if out of bounds, jumps to exit
                let item_const = self.chunk.add_constant(Constant::String(item));
                self.chunk.emit(OpCode::LoadConst(item_const));
                // JumpIfFalse placeholder for loop exit
                let exit_jump = self.chunk.emit(OpCode::JumpIfFalse(0));

                // body
                self.compile_statements(body)?;

                // jump back to loop start
                self.chunk.emit(OpCode::Jump(loop_start));

                // patch exit
                let after = self.chunk.instructions.len();
                self.chunk.patch(exit_jump, OpCode::JumpIfFalse(after));
            }

            Statement::Help => {
                let hi = self
                    .chunk
                    .add_constant(Constant::String("Help is available in the interpreter. Run `talea` without arguments.".to_string()));
                self.chunk.emit(OpCode::LoadConst(hi));
                self.chunk.emit(OpCode::Print);
            }
        }
        Ok(())
    }

    // ─── Helpers ─────────────────────────────────────────────────────────────

    fn extract_ident(&self, expr: &Expression) -> Result<String, String> {
        match expr {
            Expression::Identifier(s) => Ok(s.clone()),
            Expression::StringLiteral(s) => Ok(s.clone()),
            other => Err(format!("Expected identifier, got {:?}", other)),
        }
    }

    /// Pop the last emitted LoadConst and return the string it carried.
    #[allow(dead_code)]
    fn pop_name_const(&mut self) -> Result<String, String> {
        if let Some(OpCode::LoadConst(idx)) = self.chunk.instructions.last().cloned() {
            self.chunk.instructions.pop();
            if let Some(Constant::String(s)) = self.chunk.constants.get(idx) {
                return Ok(s.clone());
            }
        }
        Err("Expected name constant at top of emission stream".to_string())
    }

    pub fn finish(self) -> Chunk {
        self.chunk
    }
}

fn backend_name(b: &Backend) -> String {
    match b {
        Backend::Python => "python".to_string(),
        Backend::R => "r".to_string(),
        Backend::Java => "java".to_string(),
    }
}

/// Public entry point.
pub fn compile(statements: Vec<Statement>) -> Result<Chunk, String> {
    let mut compiler = Compiler::new();
    compiler.compile_statements(statements)?;
    Ok(compiler.finish())
}
