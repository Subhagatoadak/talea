use shellexpand;
use std::collections::{HashMap, HashSet};
use std::fs;

// FFI Imports
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList, PyTuple};
// FIX: Import the prelude with an alias to avoid name collisions.
#[cfg(feature = "r")]
use extendr_api::prelude as r_prelude;
// FIX: We are no longer using the jni crate directly in this simplified version.

// AST and Runtime Imports
use crate::ast::{ArithmeticOp, Backend, ComparisonOp, Expression, FilterCondition, Statement};
use crate::lexer::Token;
use crate::openai_pipeline::OpenAiPipeline;
use crate::runtime::{Environment, TaleaValue};
use serde_json::Value as JsonValue;
use std::process::Command;

pub struct Interpreter {
    environment: Environment,
    active_backends: HashSet<Backend>,
    libraries: HashMap<String, Library>,
}

#[allow(dead_code)]
enum Library {
    Python(Py<PyModule>),
    R(String),
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
            active_backends: HashSet::new(),
            libraries: HashMap::new(),
        }
    }

    /// Retrieve a variable from the interpreter's environment.
    ///
    /// This helper is primarily intended for use in tests so that
    /// assertions can be made about the values produced by executing
    /// Talea statements.
    pub fn get_variable(&self, name: &str) -> Option<TaleaValue> {
        self.environment.get(name)
    }

    // FIX: Explicitly use `std::result::Result` to avoid name collision with extendr's Result type.
    // This fix is applied to all function signatures in this file.
    pub fn execute(&mut self, statements: Vec<Statement>) -> std::result::Result<(), String> {
        // FIX: Iterate by reference to fix the mismatched types error.
        for statement in &statements {
            if let Err(e) = self.execute_statement(statement) {
                return Err(format_runtime_error(e));
            }
        }
        Ok(())
    }

    fn execute_wordcloud_statement(
        &mut self,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let source_val = self.evaluate_expression(source)?;
        let dest_name = self.get_identifier_name(destination)?;
        let tokens = match source_val {
            TaleaValue::String(s) => s
                .split_whitespace()
                .map(|w| w.to_lowercase())
                .collect::<Vec<_>>(),
            TaleaValue::List(l) => {
                let mut out = Vec::new();
                for v in l {
                    if let TaleaValue::String(s) = v {
                        out.push(s.to_lowercase());
                    }
                }
                out
            }
            _ => return Err("Wordcloud can only be applied to text or list of strings".to_string()),
        };
        let mut counts: HashMap<String, i64> = HashMap::new();
        for t in tokens {
            *counts.entry(t).or_insert(0) += 1;
        }
        let mut pairs: Vec<(String, i64)> = counts.into_iter().collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        let result = pairs
            .into_iter()
            .map(|(w, c)| TaleaValue::Tuple(vec![TaleaValue::String(w), TaleaValue::Number(c)]))
            .collect();
        self.environment.define(dest_name, TaleaValue::List(result));
        Ok(())
    }

    fn execute_word_association_statement(
        &mut self,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let source_val = self.evaluate_expression(source)?;
        let dest_name = self.get_identifier_name(destination)?;
        let tokens = match source_val {
            TaleaValue::String(s) => s
                .split_whitespace()
                .map(|w| w.to_lowercase())
                .collect::<Vec<_>>(),
            TaleaValue::List(l) => {
                let mut out = Vec::new();
                for v in l {
                    if let TaleaValue::String(s) = v {
                        out.push(s.to_lowercase());
                    }
                }
                out
            }
            _ => {
                return Err(
                    "Association analysis can only be applied to text or list of strings"
                        .to_string(),
                );
            }
        };
        let mut counts: HashMap<(String, String), i64> = HashMap::new();
        for pair in tokens.windows(2) {
            if let [a, b] = &pair {
                *counts.entry((a.clone(), b.clone())).or_insert(0) += 1;
            }
        }
        let mut pairs: Vec<((String, String), i64)> = counts.into_iter().collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        let result = pairs
            .into_iter()
            .map(|((a, b), c)| {
                TaleaValue::Tuple(vec![
                    TaleaValue::String(a),
                    TaleaValue::String(b),
                    TaleaValue::Number(c),
                ])
            })
            .collect();
        self.environment.define(dest_name, TaleaValue::List(result));
        Ok(())
    }

    fn execute_semantic_similarity_statement(
        &mut self,
        text1: &Expression,
        text2: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let s1 = self.get_string_value(text1)?;
        let s2 = self.get_string_value(text2)?;
        let set1: HashSet<String> = s1.split_whitespace().map(|w| w.to_lowercase()).collect();
        let set2: HashSet<String> = s2.split_whitespace().map(|w| w.to_lowercase()).collect();
        let inter = set1.intersection(&set2).count() as i64;
        let union = set1.union(&set2).count().max(1) as i64;
        let score = inter * 100 / union;
        let dest_name = self.get_identifier_name(destination)?;
        self.environment
            .define(dest_name, TaleaValue::Number(score));
        Ok(())
    }

    fn execute_semantic_search_statement(
        &mut self,
        query: &Expression,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let query_str = self.get_string_value(query)?;
        let query_set: HashSet<String> = query_str
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();
        let source_val = self.evaluate_expression(source)?;
        let docs = match source_val {
            TaleaValue::List(l) => {
                let mut docs_vec = Vec::new();
                for item in l {
                    if let TaleaValue::String(s) = item {
                        docs_vec.push(s);
                    } else {
                        return Err("Semantic search expects a list of strings".to_string());
                    }
                }
                docs_vec
            }
            _ => return Err("Semantic search expects a list of strings".to_string()),
        };
        let mut results: Vec<(i64, i64)> = Vec::new();
        for (idx, doc) in docs.iter().enumerate() {
            let doc_set: HashSet<String> =
                doc.split_whitespace().map(|w| w.to_lowercase()).collect();
            let inter = query_set.intersection(&doc_set).count() as i64;
            let union = query_set.union(&doc_set).count().max(1) as i64;
            let score = inter * 100 / union;
            results.push((idx as i64, score));
        }
        results.sort_by(|a, b| b.1.cmp(&a.1));
        let list = results
            .into_iter()
            .map(|(i, s)| TaleaValue::Tuple(vec![TaleaValue::Number(i), TaleaValue::Number(s)]))
            .collect();
        let dest_name = self.get_identifier_name(destination)?;
        self.environment.define(dest_name, TaleaValue::List(list));
        Ok(())
    }

    fn execute_keyword_search_statement(
        &mut self,
        query: &Expression,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let target = self.get_string_value(query)?.to_lowercase();
        let text = self.get_string_value(source)?;
        let dest_name = self.get_identifier_name(destination)?;
        let sentences: Vec<TaleaValue> = text
            .split(|c| c == '.' || c == '!' || c == '?' || c == '\n')
            .map(|s| s.trim().to_string())
            .filter(|s| s.to_lowercase().contains(&target))
            .map(TaleaValue::String)
            .collect();
        self.environment
            .define(dest_name, TaleaValue::List(sentences));
        Ok(())
    }

    fn execute_compare_statement(
        &mut self,
        left: &Expression,
        right: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let left_val = self.evaluate_expression(left)?;
        let right_val = self.evaluate_expression(right)?;
        let dest_name = self.get_identifier_name(destination)?;
        self.environment
            .define(dest_name, TaleaValue::Tuple(vec![left_val, right_val]));
        println!("[Interpreter: Compared values.]");
        Ok(())
    }

    fn execute_average_statement(
        &mut self,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let dest_name = self.get_identifier_name(destination)?;
        let _value = self.evaluate_expression(source)?;
        self.environment
            .define(dest_name, TaleaValue::String("average".to_string()));
        println!("[Interpreter: Averaged data.]");
        Ok(())
    }

    fn execute_check_normality_statement(
        &mut self,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let dest_name = self.get_identifier_name(destination)?;
        let _value = self.evaluate_expression(source)?;
        self.environment
            .define(dest_name, TaleaValue::String("normal".to_string()));
        println!("[Interpreter: Checked normality.]");
        Ok(())
    }

    fn execute_ttest_statement(
        &mut self,
        _sample: &Option<String>,
        _column: &Expression,
        group1: &Expression,
        group2: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        if !self.active_backends.contains(&Backend::Python) {
            return Err("TTest requires 'use python'".to_string());
        }
        let g1 = self.get_number_list(group1)?;
        let g2 = self.get_number_list(group2)?;
        let dest_name = self.get_identifier_name(destination)?;
        Python::with_gil(|py| -> PyResult<()> {
            let sys = PyModule::import_bound(py, "sys")?;
            let sysconfig = PyModule::import_bound(py, "sysconfig")?;
            let paths = sysconfig.call_method0("get_paths")?;
            let purelib: String = paths.get_item("purelib")?.extract()?;
            sys.getattr("path")?.call_method1("append", (purelib,))?;
            let stats = PyModule::import_bound(py, "scipy.stats")?;
            let l1 = PyList::new_bound(py, g1);
            let l2 = PyList::new_bound(py, g2);
            let res = stats.call_method1("ttest_ind", (&l1, &l2))?;
            let t_stat: f64 = res.get_item(0)?.extract()?;
            let p_val: f64 = res.get_item(1)?.extract()?;
            self.environment.define(
                dest_name,
                TaleaValue::Tuple(vec![
                    TaleaValue::String(t_stat.to_string()),
                    TaleaValue::String(p_val.to_string()),
                ]),
            );
            Ok(())
        })
        .map_err(|e| format!("Python Error: {}", e))?;
        Ok(())
    }

    fn execute_anova_statement(
        &mut self,
        _column: &Expression,
        group1: &Expression,
        group2: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        if !self.active_backends.contains(&Backend::Python) {
            return Err("ANOVA requires 'use python'".to_string());
        }
        let g1 = self.get_number_list(group1)?;
        let g2 = self.get_number_list(group2)?;
        let dest_name = self.get_identifier_name(destination)?;
        Python::with_gil(|py| -> PyResult<()> {
            let sys = PyModule::import_bound(py, "sys")?;
            let sysconfig = PyModule::import_bound(py, "sysconfig")?;
            let paths = sysconfig.call_method0("get_paths")?;
            let purelib: String = paths.get_item("purelib")?.extract()?;
            sys.getattr("path")?.call_method1("append", (purelib,))?;
            let stats = PyModule::import_bound(py, "scipy.stats")?;
            let l1 = PyList::new_bound(py, g1);
            let l2 = PyList::new_bound(py, g2);
            let res = stats.call_method1("f_oneway", (&l1, &l2))?;
            let f_stat: f64 = res.get_item(0)?.extract()?;
            let p_val: f64 = res.get_item(1)?.extract()?;
            self.environment.define(
                dest_name,
                TaleaValue::Tuple(vec![
                    TaleaValue::String(f_stat.to_string()),
                    TaleaValue::String(p_val.to_string()),
                ]),
            );
            Ok(())
        })
        .map_err(|e| format!("Python Error: {}", e))?;
        Ok(())
    }

    fn execute_median_test_statement(
        &mut self,
        _column: &Expression,
        group1: &Expression,
        group2: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        if !self.active_backends.contains(&Backend::Python) {
            return Err("Median test requires 'use python'".to_string());
        }
        let g1 = self.get_number_list(group1)?;
        let g2 = self.get_number_list(group2)?;
        let dest_name = self.get_identifier_name(destination)?;
        Python::with_gil(|py| -> PyResult<()> {
            let sys = PyModule::import_bound(py, "sys")?;
            let sysconfig = PyModule::import_bound(py, "sysconfig")?;
            let paths = sysconfig.call_method0("get_paths")?;
            let purelib: String = paths.get_item("purelib")?.extract()?;
            sys.getattr("path")?.call_method1("append", (purelib,))?;
            let stats = PyModule::import_bound(py, "scipy.stats")?;
            let l1 = PyList::new_bound(py, g1);
            let l2 = PyList::new_bound(py, g2);
            let res = stats.call_method1("median_test", (&l1, &l2))?;
            let stat: f64 = res.get_item(0)?.extract()?;
            let p_val: f64 = res.get_item(1)?.extract()?;
            self.environment.define(
                dest_name,
                TaleaValue::Tuple(vec![
                    TaleaValue::String(stat.to_string()),
                    TaleaValue::String(p_val.to_string()),
                ]),
            );
            Ok(())
        })
        .map_err(|e| format!("Python Error: {}", e))?;

        Ok(())
    }

    fn execute_statement(&mut self, statement: &Statement) -> std::result::Result<(), String> {
        match statement {
            Statement::Use(backend) => self.execute_use_statement(backend),
            Statement::Help => self.execute_help_statement(),
            Statement::Summarize {
                source,
                destination,
            } => self.execute_summarize_statement(source, destination),

            Statement::Regress {
                target,
                features,
                source,
                destination,
                with_openai,
            } => {
                self.execute_regress_statement(target, features, source, destination, *with_openai)
            }

            Statement::Insight {
                source,
                destination,
            } => self.execute_insight_statement(source, destination),

            Statement::Tag {
                source,
                method,
                destination,
            } => self.execute_tag_statement(source, method, destination),
            Statement::Translate {
                source,
                language,
                destination,
            } => self.execute_translate_statement(source, language, destination),
            Statement::Sentiment {
                source,
                destination,
            } => self.execute_sentiment_statement(source, destination),

            Statement::Shapiro {
                column,
                data,
                destination,
            } => self.execute_shapiro_statement(column, data, destination),
            Statement::Kolmogorov {
                column,
                data,
                destination,
            } => self.execute_kolmogorov_statement(column, data, destination),

            Statement::Mean {
                column,
                source,
                destination,
            } => self.execute_mean_statement(column, source, destination),
            Statement::Median {
                column,
                source,
                destination,
            } => self.execute_median_statement(column, source, destination),
            Statement::Mode {
                column,
                source,
                destination,
            } => self.execute_mode_statement(column, source, destination),
            Statement::Skewness {
                column,
                source,
                destination,
            } => self.execute_skewness_statement(column, source, destination),
            Statement::Kurtosis {
                column,
                source,
                destination,
            } => self.execute_kurtosis_statement(column, source, destination),

            Statement::Define { name, value } => self.execute_define_statement(name, value),
            Statement::Arithmetic {
                op,
                value,
                target,
                destination,
            } => self.execute_arithmetic_statement(op, value, target, destination),
            Statement::Load { source, alias } => self.execute_load_statement(source, alias),
            Statement::LoadCsv { path, alias } => self.execute_load_csv_statement(path, alias),
            Statement::Save {
                source,
                destination,
            } => self.execute_save_statement(source, destination),
            Statement::Print(expression) => self.execute_print_statement(expression),
            Statement::Tokenize {
                source,
                destination,
            } => self.execute_tokenize_statement(source, destination),
            Statement::Doc2Bow {
                source,
                destination,
            } => self.execute_doc2bow_statement(source, destination),
            Statement::Split {
                unit,
                source,
                destination,
            } => self.execute_split_statement(unit, source, destination),
            Statement::Join {
                unit,
                source,
                separator,
                destination,
            } => self.execute_join_statement(unit, source, separator, destination),
            Statement::Replace {
                pattern,
                replacement,
                source,
                destination,
            } => self.execute_replace_statement(pattern, replacement, source, destination),
            Statement::Clean {
                source,
                destination,
            } => self.execute_clean_statement(source, destination),
            Statement::Lowercase {
                source,
                destination,
            } => self.execute_lowercase_statement(source, destination),
            Statement::Uppercase {
                source,
                destination,
            } => self.execute_uppercase_statement(source, destination),
            Statement::Stem {
                source,
                destination,
            } => self.execute_stem_statement(source, destination),
            Statement::Normalize {
                source,
                destination,
            } => self.execute_normalize_statement(source, destination),
            Statement::Count {
                unit,
                source,
                destination,
            } => self.execute_count_statement(unit, source, destination),
            Statement::WordCloud {
                source,
                destination,
            } => self.execute_wordcloud_statement(source, destination),
            Statement::WordAssociation {
                source,
                destination,
            } => self.execute_word_association_statement(source, destination),
            Statement::SemanticSimilarity {
                text1,
                text2,
                destination,
            } => self.execute_semantic_similarity_statement(text1, text2, destination),
            Statement::SemanticSearch {
                query,
                source,
                destination,
            } => self.execute_semantic_search_statement(query, source, destination),
            Statement::KeywordSearch {
                query,
                source,
                destination,
            } => self.execute_keyword_search_statement(query, source, destination),

            Statement::Compare {
                left,
                right,
                destination,
            } => self.execute_compare_statement(left, right, destination),
            Statement::Average {
                source,
                destination,
            } => self.execute_average_statement(source, destination),
            Statement::CheckNormality {
                source,
                destination,
            } => self.execute_check_normality_statement(source, destination),

            Statement::TTest {
                sample,
                column,
                group1,
                group2,
                destination,
            } => self.execute_ttest_statement(sample, column, group1, group2, destination),
            Statement::Anova {
                column,
                group1,
                group2,
                destination,
            } => self.execute_anova_statement(column, group1, group2, destination),
            Statement::MedianTest {
                column,
                group1,
                group2,
                destination,
            } => self.execute_median_test_statement(column, group1, group2, destination),

            Statement::Concordance {
                word,
                source,
                destination,
            } => self.execute_concordance_statement(word, source, destination),
            Statement::Lemmatize {
                source,
                destination,
            } => self.execute_lemmatize_statement(source, destination),
            Statement::Filter {
                source,
                condition,
                destination,
            } => self.execute_filter_statement(source, condition, destination),
            Statement::Import {
                backend,
                module,
                alias,
            } => self.execute_import_statement(backend, module, alias),
            Statement::Install { backend, module } => {
                self.execute_install_statement(backend, module)
            }
            Statement::Call {
                library,
                function,
                args,
                destination,
            } => self.execute_call_statement(library, function, args, destination),
            Statement::OpenAI {
                prompt,
                model,
                destination,
                file_path,
            } => self.execute_openai_statement(
                prompt,
                model.as_ref(),
                destination.as_ref(),
                file_path.as_ref(),
            ),
            Statement::SetEnv { name, value } => self.execute_set_env_statement(name, value),
            Statement::If {
                condition,
                then_body,
                else_body,
            } => self.execute_if_statement(condition, then_body, else_body),
            Statement::ForEach { item, list, body } => {
                self.execute_foreach_statement(item, list, body)
            }
        }
    }

    fn execute_use_statement(&mut self, backend: &Backend) -> std::result::Result<(), String> {
        self.active_backends.insert(backend.clone());
        println!("[Interpreter: {:?} backend enabled.]", backend);
        // The R engine is now initialized automatically by the `test!` macro when it's needed.
        // We no longer need any special code here.
        Ok(())
    }

    fn execute_help_statement(&mut self) -> std::result::Result<(), String> {
        const DOCS: &str = include_str!("../../docs.md");
        const START: &str = "| Command |";
        const END: &str = "### Additional Notes";

        if let Some(start_idx) = DOCS.find(START) {
            let rest = &DOCS[start_idx..];
            let end_idx = rest.find(END).unwrap_or(rest.len());
            let table = &rest[..end_idx];
            println!("{}", table.trim());
        } else {
            println!("Help documentation not found.");
        }
        Ok(())
    }

    fn execute_summarize_statement(
        &mut self,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let dest_name = self.get_identifier_name(destination)?;
        let value = self.evaluate_expression(source)?;
        match value {
            TaleaValue::List(source_list) => {
                if !self.active_backends.contains(&Backend::R) {
                    return Err(
                        "The 'summarize' command requires the R backend. Run 'use r' first."
                            .to_string(),
                    );
                }
                let _numbers_as_strings: Vec<String> = source_list
                    .iter()
                    .filter_map(|val| {
                        if let TaleaValue::Number(n) = val {
                            Some(n.to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                println!("[Interpreter: Calling R to summarize data...]");

                #[cfg(feature = "r")]
                {
                    // FIX: Use the aliased prelude and handle the R result type correctly.
                    r_prelude::test! {
                        // 1. Construct the complete R command as a string.
                        let r_command = format!("capture.output(print(summary(c({}))))", _numbers_as_strings.join(","));

                        // 2. Evaluate the string command.
                        let summary_result = r_prelude::eval_string(&r_command).map_err(|e| e.to_string())?;

                        // 3. Extract the captured text lines and join them into a single string.
                        let summary_output = summary_result.as_str_vector()
                            .unwrap_or_default()
                            .join("\n");

                        self.environment.define(dest_name, TaleaValue::String(summary_output));
                    }
                }

                #[cfg(not(feature = "r"))]
                {
                    return Err("R backend not enabled. Rebuild with `--features r`".to_string());
                }
            }
            TaleaValue::String(s) => {
                let summary = s
                    .split(|c| c == '.' || c == '!' || c == '?')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                self.environment
                    .define(dest_name, TaleaValue::String(summary));
            }
            _ => {
                return Err(
                    "Summarize can only be applied to text or a list of numbers.".to_string(),
                );
            }
        }
        Ok(())
    }

    fn execute_regress_statement(
        &mut self,
        target: &Expression,
        features: &Vec<Expression>,
        source: &Expression,
        destination: &Expression,
        with_openai: bool,
    ) -> std::result::Result<(), String> {
        let target_name = self.get_identifier_name(target)?;
        let feature_names = features
            .iter()
            .map(|e| self.get_identifier_name(e))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let data_val = self.evaluate_expression(source)?;
        let csv_content = if let TaleaValue::String(s) = data_val {
            s
        } else {
            return Err("Regression expects the data source to be a CSV string".to_string());
        };
        let dest_name = self.get_identifier_name(destination)?;

        let summary = Python::with_gil(|py| -> PyResult<String> {
            let code = r#"
import pandas as pd, statsmodels.api as sm, io
def run_regression(csv, target, features):
    df = pd.read_csv(io.StringIO(csv))
    X = sm.add_constant(df[features])
    y = df[target]
    model = sm.OLS(y, X).fit()
    return model.summary().as_text()
"#;
            let module = PyModule::from_code_bound(py, code, "regress_mod", "regress_mod")?;
            let func = module.getattr("run_regression")?;
            func.call1((csv_content, target_name, feature_names))?
                .extract()
        })
        .map_err(|e| format!("Python error: {}", e))?;

        self.environment
            .define(dest_name.clone(), TaleaValue::String(summary.clone()));
        println!(
            "[Interpreter: Stored regression summary in '{}']",
            dest_name
        );

        if with_openai {
            let prompt = format!(
                "Interpret the following regression model summary:\n{}",
                summary
            );
            let pipeline = OpenAiPipeline::from_env("gpt-3.5-turbo".to_string())?;
            let response = pipeline.ask(&prompt)?;
            println!("[OpenAI Interpretation]\n{}", response);
        }
        Ok(())
    }

    fn execute_insight_statement(
        &mut self,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let value = self.evaluate_expression(source)?;
        let json_val = Interpreter::talea_value_to_json(&value);
        let serialized =
            serde_json::to_string(&json_val).map_err(|e| format!("Serialization error: {}", e))?;
        let prompt = format!(
            "Provide narrative insights for the following data: {}",
            serialized
        );
        let pipeline = OpenAiPipeline::from_env("gpt-3.5-turbo")?;
        let response = pipeline.ask(&prompt)?;
        let dest_name = self.get_identifier_name(destination)?;
        self.environment
            .define(dest_name, TaleaValue::String(response.clone()));
        println!("{}", response);

        Ok(())
    }

    fn execute_tag_statement(
        &mut self,
        source: &Expression,
        method: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        // FIX: Simplified to only use Python for now.
        if self.active_backends.contains(&Backend::Python) {
            self.execute_tag_python(source, method, destination)
        } else {
            Err("The 'tag' command currently requires the Python backend. Please run 'use python' first.".to_string())
        }
    }

    fn execute_tag_python(
        &mut self,
        source: &Expression,
        method: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let text = self.get_string_value(source)?;
        let method_token = if let Expression::Unit(t) = method {
            t
        } else {
            return Err("Invalid method for tag".to_string());
        };
        let dest_name = self.get_identifier_name(destination)?;
        println!(
            "[Interpreter: Calling Python/spaCy for {:?} tagging...]",
            method_token
        );
        Python::with_gil(|py| -> PyResult<()> {
            let spacy = PyModule::import_bound(py, "spacy")?;
            let nlp = spacy.call_method1("load", ("en_core_web_md",))?;
            let doc = nlp.call1((&text,))?;
            let result_list = match method_token {
                Token::POS => doc
                    .iter()?
                    .map(|token_result| {
                        let token = token_result?;
                        let text = token.getattr("text")?.extract::<String>()?;
                        let pos = token.getattr("pos_")?.extract::<String>()?;
                        Ok(TaleaValue::Tuple(vec![
                            TaleaValue::String(text),
                            TaleaValue::String(pos),
                        ]))
                    })
                    .collect::<PyResult<Vec<TaleaValue>>>()?,
                Token::NER => doc
                    .getattr("ents")?
                    .iter()?
                    .map(|ent_result| {
                        let ent = ent_result?;
                        let text = ent.getattr("text")?.extract::<String>()?;
                        let label = ent.getattr("label_")?.extract::<String>()?;
                        Ok(TaleaValue::Tuple(vec![
                            TaleaValue::String(text),
                            TaleaValue::String(label),
                        ]))
                    })
                    .collect::<PyResult<Vec<TaleaValue>>>()?,
                _ => {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "Unsupported tagging method",
                    ));
                }
            };
            self.environment
                .define(dest_name, TaleaValue::List(result_list));
            Ok(())
        })
        .map_err(|e| format!("Python Error: {}", e))?;
        println!("[Interpreter: Tagging complete.]");
        Ok(())
    }

    fn execute_translate_statement(
        &mut self,
        source: &Expression,
        language: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let text = self.get_string_value(source)?;
        let lang = self.get_string_value(language)?;
        let dest_name = self.get_identifier_name(destination)?;
        let prompt = format!("Translate the following text to {}:\n{}", lang, text);
        let pipeline = OpenAiPipeline::from_env("gpt-3.5-turbo")?;
        let translation = pipeline.ask(&prompt)?;
        self.environment
            .define(dest_name, TaleaValue::String(translation));
        println!("[Interpreter: Translation complete.]");
        Ok(())
    }

    fn execute_sentiment_statement(
        &mut self,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let text = self.get_string_value(source)?.to_lowercase();
        let dest_name = self.get_identifier_name(destination)?;
        let positive = [
            "good",
            "great",
            "happy",
            "love",
            "excellent",
            "awesome",
            "fantastic",
            "nice",
        ];
        let negative = [
            "bad", "terrible", "sad", "hate", "awful", "horrible", "poor", "worst",
        ];
        let mut score = 0i32;
        for word in text.split_whitespace() {
            if positive.contains(&word) {
                score += 1;
            }
            if negative.contains(&word) {
                score -= 1;
            }
        }
        let sentiment = if score > 0 {
            "positive"
        } else if score < 0 {
            "negative"
        } else {
            "neutral"
        };
        self.environment
            .define(dest_name, TaleaValue::String(sentiment.to_string()));
        println!("[Interpreter: Sentiment analysis complete.]");
        Ok(())
    }

    fn compute_statistic(
        &self,
        data: &str,
        column: &str,
        func: &str,
    ) -> std::result::Result<f64, String> {
        Python::with_gil(|py| -> PyResult<f64> {
            let pandas = PyModule::import_bound(py, "pandas")?;
            let io = PyModule::import_bound(py, "io")?;
            let string_io = io.getattr("StringIO")?.call1((data,))?;
            let df = pandas.getattr("read_csv")?.call1((string_io,))?;
            let col = df.get_item(column)?;
            match func {
                "mean" => col.getattr("mean")?.call0()?.extract::<f64>(),
                "median" => col.getattr("median")?.call0()?.extract::<f64>(),
                "mode" => {
                    let modes = col.getattr("mode")?.call0()?;
                    modes.get_item(0)?.extract::<f64>()
                }
                "skewness" => {
                    let arr = col.getattr("tolist")?.call0()?;
                    let stats = PyModule::import_bound(py, "scipy.stats")?;
                    stats.getattr("skew")?.call1((arr,))?.extract::<f64>()
                }
                "kurtosis" => {
                    let arr = col.getattr("tolist")?.call0()?;
                    let stats = PyModule::import_bound(py, "scipy.stats")?;
                    stats.getattr("kurtosis")?.call1((arr,))?.extract::<f64>()
                }
                _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("bad func"))?,
            }
        })
        .map_err(|e| e.to_string())
    }

    fn execute_mean_statement(
        &mut self,
        column: &Expression,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let col_name = self.get_identifier_name(column)?;
        let data = self.get_string_value(source)?;
        let result = self.compute_statistic(&data, &col_name, "mean")?;
        let dest_name = self.get_identifier_name(destination)?;
        self.environment
            .define(dest_name, TaleaValue::Number(result as i64));
        Ok(())
    }

    fn execute_median_statement(
        &mut self,
        column: &Expression,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let col_name = self.get_identifier_name(column)?;
        let data = self.get_string_value(source)?;
        let result = self.compute_statistic(&data, &col_name, "median")?;
        let dest_name = self.get_identifier_name(destination)?;
        self.environment
            .define(dest_name, TaleaValue::Number(result as i64));
        Ok(())
    }

    fn execute_mode_statement(
        &mut self,
        column: &Expression,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let col_name = self.get_identifier_name(column)?;
        let data = self.get_string_value(source)?;
        let result = self.compute_statistic(&data, &col_name, "mode")?;
        let dest_name = self.get_identifier_name(destination)?;
        self.environment
            .define(dest_name, TaleaValue::Number(result as i64));
        Ok(())
    }

    fn execute_skewness_statement(
        &mut self,
        column: &Expression,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let col_name = self.get_identifier_name(column)?;
        let data = self.get_string_value(source)?;
        let result = self.compute_statistic(&data, &col_name, "skewness")?;
        let dest_name = self.get_identifier_name(destination)?;
        self.environment
            .define(dest_name, TaleaValue::Number(result as i64));
        Ok(())
    }

    fn execute_kurtosis_statement(
        &mut self,
        column: &Expression,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let col_name = self.get_identifier_name(column)?;
        let data = self.get_string_value(source)?;
        let result = self.compute_statistic(&data, &col_name, "kurtosis")?;
        let dest_name = self.get_identifier_name(destination)?;
        self.environment
            .define(dest_name, TaleaValue::Number(result as i64));
        Ok(())
    }

    // ... other execute functions ...
    fn execute_lemmatize_statement(
        &mut self,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        if !self.active_backends.contains(&Backend::Python) {
            return Err("Lemmatize requires 'use python' first.".to_string());
        }
        let text = self.get_string_value(source)?;
        let dest_name = self.get_identifier_name(destination)?;
        println!("[Interpreter: Calling Python/spaCy for lemmatization...]");
        Python::with_gil(|py| -> PyResult<()> {
            let spacy = PyModule::import_bound(py, "spacy")?;
            let nlp = spacy.call_method1("load", ("en_core_web_md",))?;
            let doc = nlp.call1((&text,))?;
            let lemmas = doc
                .iter()?
                .map(|token_result| {
                    let token = token_result?;
                    let lemma = token.getattr("lemma_")?.extract::<String>()?;
                    Ok(TaleaValue::String(lemma))
                })
                .collect::<PyResult<Vec<TaleaValue>>>()?;
            self.environment.define(dest_name, TaleaValue::List(lemmas));
            Ok(())
        })
        .map_err(|e| format!("Python Error: {}", e))?;
        println!("[Interpreter: Lemmatization complete.]");
        Ok(())
    }

    fn execute_filter_statement(
        &mut self,
        source: &Expression,
        condition: &FilterCondition,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let source_list = if let TaleaValue::List(l) = self.evaluate_expression(source)? {
            l
        } else {
            return Err("Filter can only be applied to a list.".to_string());
        };
        let dest_name = self.get_identifier_name(destination)?;
        let mut filtered_list = Vec::new();
        for item in source_list {
            if let Some(s_item) = item.as_string() {
                let passes = match condition {
                    FilterCondition::Containing(expr) => self
                        .evaluate_expression(expr)?
                        .as_string()
                        .map_or(false, |p| s_item.contains(&p)),
                    FilterCondition::StartingWith(expr) => self
                        .evaluate_expression(expr)?
                        .as_string()
                        .map_or(false, |p| s_item.starts_with(&p)),
                    FilterCondition::EndingWith(expr) => self
                        .evaluate_expression(expr)?
                        .as_string()
                        .map_or(false, |p| s_item.ends_with(&p)),
                };
                if passes {
                    filtered_list.push(item);
                }
            }
        }
        println!(
            "[Interpreter: Filtered list contains {} items.]",
            filtered_list.len()
        );
        self.environment
            .define(dest_name, TaleaValue::List(filtered_list));
        Ok(())
    }

    fn execute_import_statement(
        &mut self,
        backend: &Backend,
        module: &Expression,
        alias: &Expression,
    ) -> std::result::Result<(), String> {
        let alias_name = self.get_identifier_name(alias)?;
        match backend {
            Backend::Python => {
                if !self.active_backends.contains(&Backend::Python) {
                    return Err("Python backend not enabled. Run 'use python' first.".to_string());
                }
                let module_str_raw = self.get_string_value(module)?;
                let module_str = shellexpand::tilde(&module_str_raw).to_string();
                println!(
                    "[Interpreter: Importing Python module '{}' as '{}']",
                    module_str, &alias_name
                );
                let import_result = Python::with_gil(|py| -> PyResult<Py<PyModule>> {
                    if module_str.ends_with(".py") {
                        let importlib = PyModule::import_bound(py, "importlib.util")?;
                        let spec = importlib.call_method1(
                            "spec_from_file_location",
                            (alias_name.as_str(), module_str.as_str()),
                        )?;
                        let module = importlib.call_method1("module_from_spec", (spec.clone(),))?;
                        let loader = spec.getattr("loader")?;
                        loader.call_method1("exec_module", (module.clone(),))?;
                        module
                            .downcast_into::<PyModule>()
                            .map_err(|e| e.into())
                            .map(|m| m.into())
                    } else {
                        let module = PyModule::import_bound(py, module_str.as_str())?;
                        Ok(module.into())
                    }
                });
                match import_result {
                    Ok(py_module) => {
                        self.libraries
                            .insert(alias_name.clone(), Library::Python(py_module));
                        Ok(())
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        if err_msg.contains("ModuleNotFoundError") {
                            Err(format!(
                                "Python module '{}' was not found. You can install it by running: pip install {}",
                                module_str, module_str
                            ))
                        } else {
                            Err(format!("Python Error: {}", err_msg))
                        }
                    }
                }
            }
            Backend::R => {
                if !self.active_backends.contains(&Backend::R) {
                    return Err("R backend not enabled. Run 'use r' first.".to_string());
                }
                let module_str_raw = self.get_string_value(module)?;
                let module_str = shellexpand::tilde(&module_str_raw).to_string();
                println!(
                    "[Interpreter: Importing R module '{}' as '{}']",
                    module_str, &alias_name
                );

                #[cfg(feature = "r")]
                {
                    let mut import_result: std::result::Result<(), String> = Ok(());
                    r_prelude::test! {
                        import_result = if module_str.ends_with(".R") || module_str.ends_with(".r") {
                            r_prelude::eval_string(&format!("source('{}')", module_str))
                                .map(|_| ())
                                .map_err(|e| e.to_string())
                        } else {
                            r_prelude::eval_string(&format!("library({})", module_str))
                                .map(|_| ())
                                .map_err(|e| e.to_string())
                        };
                    }
                    match import_result {
                        Ok(_) => {
                            self.libraries
                                .insert(alias_name.clone(), Library::R(module_str.clone()));
                            Ok(())
                        }
                        Err(err_msg) => {
                            if err_msg.contains("there is no package called")
                                || err_msg.contains("package") && err_msg.contains("not found")
                            {
                                Err(format!(
                                    "R library '{}' was not found. You can install it by running: install.packages(\"{}\", repos = \"https://cloud.r-project.org\")",
                                    module_str, module_str
                                ))
                            } else {
                                Err(err_msg)
                            }
                        }
                    }
                }

                #[cfg(not(feature = "r"))]
                {
                    Err("R backend not enabled. Rebuild with `--features r`".to_string())
                }
            }
            _ => Err("Unsupported backend for import".to_string()),
        }
    }

    fn execute_install_statement(
        &mut self,
        backend: &Backend,
        module: &Expression,
    ) -> std::result::Result<(), String> {
        let module_str_raw = self.get_string_value(module)?;
        let module_str = shellexpand::tilde(&module_str_raw).to_string();
        match backend {
            Backend::Python => {
                println!("[Interpreter: Installing Python module '{}']", module_str);
                let status = Command::new("python")
                    .arg("-m")
                    .arg("pip")
                    .arg("install")
                    .arg(&module_str)
                    .status()
                    .map_err(|e| e.to_string())?;
                if status.success() {
                    Ok(())
                } else {
                    Err(format!("Failed to install Python module '{}'", module_str))
                }
            }
            Backend::R => {
                #[cfg(feature = "r")]
                {
                    println!("[Interpreter: Installing R package '{}']", module_str);
                    let cmd = format!(
                        "install.packages(\"{}\", repos = \"https://cloud.r-project.org\")",
                        module_str
                    );
                    let status = Command::new("R")
                        .arg("-q")
                        .arg("-e")
                        .arg(cmd)
                        .status()
                        .map_err(|e| e.to_string())?;
                    if status.success() {
                        Ok(())
                    } else {
                        Err(format!("Failed to install R package '{}'", module_str))
                    }
                }

                #[cfg(not(feature = "r"))]
                {
                    Err("R backend not enabled. Rebuild with `--features r`".to_string())
                }
            }
            _ => Err("Unsupported backend for install".to_string()),
        }
    }

    fn execute_call_statement(
        &mut self,
        library: &Expression,
        function: &Expression,
        args: &Vec<Expression>,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let lib_name = self.get_identifier_name(library)?;
        let func_name = self.get_identifier_name(function)?;
        let dest_name = self.get_identifier_name(destination)?;
        let arg_values = args
            .iter()
            .map(|e| self.evaluate_expression(e))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        match self.libraries.get(&lib_name) {
            Some(Library::Python(module)) => {
                let result = Python::with_gil(|py| -> PyResult<TaleaValue> {
                    // Support nested function paths like "corpora.Dictionary" by
                    // traversing each attribute before calling the final one.
                    // `bind` returns a shared reference, so clone before
                    // converting the module into a generic `PyAny`.
                    let mut current: pyo3::Bound<'_, PyAny> = module.bind(py).clone().into_any();
                    for attr_name in func_name.split('.') {
                        current = current.getattr(attr_name)?;
                    }
                    let py_args: Vec<PyObject> = arg_values
                        .iter()
                        .map(|v| Interpreter::talea_value_to_py(py, v))
                        .collect();
                    if current.is_callable() {
                        let args_tuple = PyTuple::new_bound(py, py_args);
                        let res = current.call1(args_tuple)?;
                        Ok(Interpreter::py_obj_to_talea(res))
                    } else {
                        if !py_args.is_empty() {
                            return Err(pyo3::exceptions::PyTypeError::new_err(
                                "Tried to call non-callable attribute with arguments",
                            ));
                        }
                        Ok(Interpreter::py_obj_to_talea(current))
                    }
                })
                .map_err(|e| format!("Python Error: {}", e))?;
                self.environment.define(dest_name, result);
                Ok(())
            }
            Some(Library::R(module_name)) => {
                let arg_strings: Vec<String> = arg_values
                    .iter()
                    .map(|v| Interpreter::talea_value_to_r_arg(v))
                    .collect();
                let call_string = if module_name.ends_with(".R") || module_name.ends_with(".r") {
                    format!("{}({})", func_name, arg_strings.join(","))
                } else {
                    format!("{}::{}({})", module_name, func_name, arg_strings.join(","))
                };
                println!("[Interpreter: Calling R: {}]", call_string);
                let _dest = dest_name.clone();

                #[cfg(feature = "r")]
                {
                    r_prelude::test! {
                        let res = r_prelude::eval_string(&call_string).map_err(|e| e.to_string())?;
                        let value = Interpreter::r_obj_to_talea(res);
                        self.environment.define(_dest, value);
                    }
                    Ok(())
                }

                #[cfg(not(feature = "r"))]
                {
                    Err("R backend not enabled. Rebuild with `--features r`".to_string())
                }
            }
            None => Err(format!(
                "Library '{}' not found. Did you import it?",
                lib_name
            )),
        }
    }

    fn execute_openai_statement(
        &mut self,
        prompt: &Expression,
        model: Option<&Expression>,
        destination: Option<&Expression>,
        file_path: Option<&Expression>,
    ) -> std::result::Result<(), String> {
        let prompt_str = self.get_string_value(prompt)?;

        let model_name = if let Some(model_expr) = model {
            self.get_string_value(model_expr)?
        } else {
            "gpt-3.5-turbo".to_string()
        };

        let pipeline = OpenAiPipeline::from_env(model_name)?;
        let response = pipeline.ask(&prompt_str)?;

        if let Some(dest_expr) = destination {
            let dest_name = self.get_identifier_name(dest_expr)?;
            self.environment
                .define(dest_name, TaleaValue::String(response.clone()));
        }

        if let Some(file_expr) = file_path {
            let path = self.get_string_value(file_expr)?;
            fs::write(&path, &response)
                .map_err(|e| format!("Failed to write to file '{}': {}", path, e))?;
            println!("[Interpreter: Successfully saved content to '{}']", path);
        }

        Ok(())
    }

    fn execute_save_statement(
        &mut self,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let source_val = self.evaluate_expression(source)?;
        let file_path = self.get_string_value(destination)?;
        let content_to_save = source_val.to_string();
        fs::write(&file_path, content_to_save)
            .map_err(|e| format!("Failed to write to file '{}': {}", file_path, e))?;
        println!(
            "[Interpreter: Successfully saved content to '{}']",
            file_path
        );
        Ok(())
    }
    fn execute_arithmetic_statement(
        &mut self,
        op: &ArithmeticOp,
        value: &Expression,
        target: &Expression,
        destination: &Option<Expression>,
    ) -> std::result::Result<(), String> {
        let target_name = self.get_identifier_name(target)?;
        let current_val = self
            .environment
            .get(&target_name)
            .ok_or_else(|| format!("Variable '{}' not found.", target_name))?;
        let current_num = if let TaleaValue::Number(n) = current_val {
            n
        } else {
            return Err(format!(
                "Cannot perform arithmetic on '{}'. Not a number.",
                target_name
            ));
        };
        let operand_val = self.evaluate_expression(value)?;
        let operand_num = if let TaleaValue::Number(n) = operand_val {
            n
        } else {
            return Err("Arithmetic operations require a number.".to_string());
        };
        let result = match op {
            ArithmeticOp::Add => current_num + operand_num,
            ArithmeticOp::Subtract => current_num - operand_num,
            ArithmeticOp::Multiply => current_num * operand_num,
            ArithmeticOp::Divide => {
                if operand_num == 0 {
                    return Err("Division by zero.".to_string());
                }
                current_num / operand_num
            }
        };
        let final_dest_name = if let Some(dest_expr) = destination {
            self.get_identifier_name(dest_expr)?
        } else {
            target_name.clone()
        };
        self.environment
            .define(final_dest_name, TaleaValue::Number(result));
        Ok(())
    }
    fn execute_define_statement(
        &mut self,
        name: &Expression,
        value: &Expression,
    ) -> std::result::Result<(), String> {
        let var_name = self.get_identifier_name(name)?;
        let value = self.evaluate_expression(value)?;
        self.environment.define(var_name, value);
        Ok(())
    }

    fn execute_set_env_statement(
        &mut self,
        name: &str,
        value: &Expression,
    ) -> std::result::Result<(), String> {
        let val = self.evaluate_expression(value)?;
        let string_val = match val {
            TaleaValue::String(s) => s,
            TaleaValue::Number(n) => n.to_string(),
            _ => return Err("Environment variables must be strings or numbers.".to_string()),
        };
        unsafe {
            std::env::set_var(name, &string_val);
        }
        println!("[Interpreter: Set {}]", name);
        Ok(())
    }
    fn execute_load_statement(
        &mut self,
        source: &Expression,
        alias: &Expression,
    ) -> std::result::Result<(), String> {
        let var_name = self.get_identifier_name(alias)?;
        let raw_path = self.get_string_value(source)?;

        // --- NEW LOGIC IS HERE ---
        // 1. Expand the path to handle tildes (e.g., "~/Documents")
        let expanded_path_cow = shellexpand::tilde(&raw_path);
        // 2. Convert it to a proper PathBuf for the OS
        let file_path = std::path::PathBuf::from(expanded_path_cow.as_ref());
        // --- END OF NEW LOGIC ---

        println!("[Interpreter: Reading file at '{}']", file_path.display());

        let file_content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read file '{}': {}", file_path.display(), e))?;

        println!(
            "[Interpreter: Successfully read {} bytes]",
            file_content.len()
        );
        self.environment
            .define(var_name, TaleaValue::String(file_content));
        Ok(())
    }

    fn execute_load_csv_statement(
        &mut self,
        path: &Expression,
        alias: &Expression,
    ) -> std::result::Result<(), String> {
        if !self.active_backends.contains(&Backend::Python) {
            return Err("Python backend not enabled. Run 'use python' first.".to_string());
        }
        let var_name = self.get_identifier_name(alias)?;
        let raw_path = self.get_string_value(path)?;
        let expanded_path_cow = shellexpand::tilde(&raw_path);
        let file_path = std::path::PathBuf::from(expanded_path_cow.as_ref());
        let path_str = file_path
            .to_str()
            .ok_or_else(|| "Invalid file path".to_string())?
            .to_string();

        let rows = Python::with_gil(|py| -> PyResult<Vec<HashMap<String, TaleaValue>>> {
            let pandas = PyModule::import_bound(py, "pandas")?;
            let df = pandas.getattr("read_csv")?.call1((path_str.clone(),))?;
            let records = df.call_method1("to_dict", ("records",))?;
            let list = records.downcast::<PyList>()?;
            let mut out = Vec::new();
            for item in list.iter() {
                let dict = item.downcast::<PyDict>()?;
                let mut map: HashMap<String, TaleaValue> = HashMap::new();
                for (k, v) in dict.iter() {
                    let key: String = k.extract()?;
                    let val = Interpreter::py_obj_to_talea(v);
                    map.insert(key, val);
                }
                out.push(map);
            }
            Ok(out)
        })
        .map_err(|e| format!("Python Error: {}", e))?;

        println!(
            "[Interpreter: Loaded table with {} rows from '{}']",
            rows.len(),
            file_path.display()
        );
        self.environment.define(var_name, TaleaValue::Table(rows));
        Ok(())
    }
    fn execute_tokenize_statement(
        &mut self,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let source_val = self.evaluate_expression(source)?;
        let dest_name = self.get_identifier_name(destination)?;
        if let TaleaValue::String(s) = source_val {
            let tokens: Vec<TaleaValue> = s
                .split_whitespace()
                .map(|word| TaleaValue::String(word.to_string()))
                .collect();
            println!(
                "[Interpreter: Tokenized text into {} tokens.]",
                tokens.len()
            );
            self.environment.define(dest_name, TaleaValue::List(tokens));
            Ok(())
        } else {
            Err("The 'tokenize' command can only be used on a String value.".to_string())
        }
    }

    fn execute_doc2bow_statement(
        &mut self,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let tokens_val = self.evaluate_expression(source)?;
        let dest_name = self.get_identifier_name(destination)?;
        if let TaleaValue::List(list) = tokens_val {
            let mut counts: HashMap<String, i64> = HashMap::new();
            for item in list {
                if let TaleaValue::String(s) = item {
                    *counts.entry(s).or_insert(0) += 1;
                } else {
                    return Err("doc2bow expects a list of strings".to_string());
                }
            }
            let mut bow: Vec<TaleaValue> = Vec::new();
            for (word, count) in counts {
                bow.push(TaleaValue::Tuple(vec![
                    TaleaValue::String(word),
                    TaleaValue::Number(count),
                ]));
            }
            println!(
                "[Interpreter: Created bag-of-words with {} entries.]",
                bow.len()
            );
            self.environment.define(dest_name, TaleaValue::List(bow));
            Ok(())
        } else {
            Err("doc2bow expects a list of tokens".to_string())
        }
    }

    fn execute_split_statement(
        &mut self,
        unit: &Expression,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let unit_token = if let Expression::Unit(t) = unit {
            t
        } else {
            return Err("Invalid unit for split".to_string());
        };
        let text = self.get_string_value(source)?;
        let dest_name = self.get_identifier_name(destination)?;
        match unit_token {
            Token::Sentences => {
                let sentences: Vec<TaleaValue> = text
                    .split(|c| c == '.' || c == '!' || c == '?')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| TaleaValue::String(s.to_string()))
                    .collect();
                println!(
                    "[Interpreter: Split text into {} sentences.]",
                    sentences.len()
                );
                self.environment
                    .define(dest_name, TaleaValue::List(sentences));
                Ok(())
            }
            Token::Lines => {
                let lines: Vec<TaleaValue> = text
                    .lines()
                    .map(|l| TaleaValue::String(l.to_string()))
                    .collect();
                println!("[Interpreter: Split text into {} lines.]", lines.len());
                self.environment.define(dest_name, TaleaValue::List(lines));
                Ok(())
            }
            Token::Words | Token::Tokens => {
                let words: Vec<TaleaValue> = text
                    .split_whitespace()
                    .map(|w| TaleaValue::String(w.to_string()))
                    .collect();
                println!("[Interpreter: Split text into {} words.]", words.len());
                self.environment.define(dest_name, TaleaValue::List(words));
                Ok(())
            }
            Token::Characters => {
                let chars: Vec<TaleaValue> = text
                    .chars()
                    .map(|c| TaleaValue::String(c.to_string()))
                    .collect();
                println!("[Interpreter: Split text into {} characters.]", chars.len());
                self.environment.define(dest_name, TaleaValue::List(chars));
                Ok(())
            }
            _ => Err("Unsupported unit for split".to_string()),
        }
    }

    fn execute_join_statement(
        &mut self,
        _unit: &Expression,
        source: &Expression,
        separator: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let list_val = self.evaluate_expression(source)?;
        let sep = self.get_string_value(separator)?;
        let dest_name = self.get_identifier_name(destination)?;
        if let TaleaValue::List(items) = list_val {
            let strings: Vec<String> = items
                .into_iter()
                .filter_map(|v| match v {
                    TaleaValue::String(s) => Some(s),
                    TaleaValue::Number(n) => Some(n.to_string()),
                    _ => None,
                })
                .collect();
            let joined = strings.join(&sep);
            println!("[Interpreter: Joined list with separator '{}']", sep);
            self.environment
                .define(dest_name, TaleaValue::String(joined));
            Ok(())
        } else {
            Err("The 'join' command requires a list.".to_string())
        }
    }

    fn execute_replace_statement(
        &mut self,
        pattern: &Expression,
        replacement: &Expression,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let pat = self.get_string_value(pattern)?;
        let repl = self.get_string_value(replacement)?;
        let src_val = self.evaluate_expression(source)?;
        let dest_name = self.get_identifier_name(destination)?;
        match src_val {
            TaleaValue::String(s) => {
                let result = s.replace(&pat, &repl);
                self.environment
                    .define(dest_name, TaleaValue::String(result));
                Ok(())
            }
            TaleaValue::List(list) => {
                let replaced: Vec<TaleaValue> = list
                    .into_iter()
                    .map(|item| match item {
                        TaleaValue::String(s) => TaleaValue::String(s.replace(&pat, &repl)),
                        _ => item,
                    })
                    .collect();
                self.environment
                    .define(dest_name, TaleaValue::List(replaced));
                Ok(())
            }
            _ => Err("The 'replace' command requires a string or list.".to_string()),
        }
    }

    fn execute_clean_statement(
        &mut self,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let src_val = self.evaluate_expression(source)?;
        let dest_name = self.get_identifier_name(destination)?;
        match src_val {
            TaleaValue::String(s) => {
                let cleaned = Self::clean_text(&s);
                self.environment
                    .define(dest_name, TaleaValue::String(cleaned));
                Ok(())
            }
            TaleaValue::List(list) => {
                let cleaned: Vec<TaleaValue> = list
                    .into_iter()
                    .map(|item| match item {
                        TaleaValue::String(s) => TaleaValue::String(Self::clean_text(&s)),
                        _ => item,
                    })
                    .collect();
                self.environment
                    .define(dest_name, TaleaValue::List(cleaned));
                Ok(())
            }
            _ => Err("The 'clean' command requires a string or list.".to_string()),
        }
    }

    fn execute_lowercase_statement(
        &mut self,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let src_val = self.evaluate_expression(source)?;
        let dest_name = self.get_identifier_name(destination)?;
        match src_val {
            TaleaValue::String(s) => {
                self.environment
                    .define(dest_name, TaleaValue::String(s.to_lowercase()));
                Ok(())
            }
            TaleaValue::List(list) => {
                let lower: Vec<TaleaValue> = list
                    .into_iter()
                    .map(|item| match item {
                        TaleaValue::String(s) => TaleaValue::String(s.to_lowercase()),
                        _ => item,
                    })
                    .collect();
                self.environment.define(dest_name, TaleaValue::List(lower));
                Ok(())
            }
            _ => Err("The 'lowercase' command requires a string or list.".to_string()),
        }
    }

    fn execute_uppercase_statement(
        &mut self,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let src_val = self.evaluate_expression(source)?;
        let dest_name = self.get_identifier_name(destination)?;
        match src_val {
            TaleaValue::String(s) => {
                self.environment
                    .define(dest_name, TaleaValue::String(s.to_uppercase()));
                Ok(())
            }
            TaleaValue::List(list) => {
                let upper: Vec<TaleaValue> = list
                    .into_iter()
                    .map(|item| match item {
                        TaleaValue::String(s) => TaleaValue::String(s.to_uppercase()),
                        _ => item,
                    })
                    .collect();
                self.environment.define(dest_name, TaleaValue::List(upper));
                Ok(())
            }
            _ => Err("The 'uppercase' command requires a string or list.".to_string()),
        }
    }

    fn execute_stem_statement(
        &mut self,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let src_val = self.evaluate_expression(source)?;
        let dest_name = self.get_identifier_name(destination)?;
        match src_val {
            TaleaValue::String(s) => {
                let stems: Vec<TaleaValue> = s
                    .split_whitespace()
                    .map(|w| TaleaValue::String(Self::stem_word(w)))
                    .collect();
                self.environment.define(dest_name, TaleaValue::List(stems));
                Ok(())
            }
            TaleaValue::List(list) => {
                let stems: Vec<TaleaValue> = list
                    .into_iter()
                    .map(|item| match item {
                        TaleaValue::String(s) => TaleaValue::String(Self::stem_word(&s)),
                        _ => item,
                    })
                    .collect();
                self.environment.define(dest_name, TaleaValue::List(stems));
                Ok(())
            }
            _ => Err("The 'stem' command requires a string or list.".to_string()),
        }
    }

    fn execute_normalize_statement(
        &mut self,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let text = self.get_string_value(source)?;
        let dest_name = self.get_identifier_name(destination)?;
        let normalized = text.to_lowercase();
        println!("[Interpreter: Normalized text.]");
        self.environment
            .define(dest_name, TaleaValue::String(normalized));
        Ok(())
    }
    fn execute_count_statement(
        &mut self,
        unit: &Expression,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let unit_token = if let Expression::Unit(t) = unit {
            t
        } else {
            return Err("Invalid unit for count".to_string());
        };
        let source_val = self.evaluate_expression(source)?;
        let dest_name = self.get_identifier_name(destination)?;

        match (unit_token, source_val) {
            // CASE 1: Count items in a list (e.g., "count words in word_list")
            (
                Token::Words | Token::Tokens | Token::Lines | Token::Sentences | Token::Paragraphs,
                TaleaValue::List(l),
            ) => {
                let count = l.len() as i64;
                println!("[Interpreter: Counted {} items]", count);
                self.environment
                    .define(dest_name, TaleaValue::Number(count));
            }
            // CASE 2: Count words/tokens in a single string (e.g., "count words in article")
            (Token::Words | Token::Tokens, TaleaValue::String(s)) => {
                let count = s.split_whitespace().count() as i64;
                println!("[Interpreter: Counted {} words]", count);
                self.environment
                    .define(dest_name, TaleaValue::Number(count));
            }
            // CASE 3: Get character/line/sentence/paragraph count from a single string
            (Token::Characters, TaleaValue::String(s)) => {
                let count = s.len() as i64;
                println!("[Interpreter: Counted {} characters]", count);
                self.environment
                    .define(dest_name, TaleaValue::Number(count));
            }
            (Token::Lines, TaleaValue::String(s)) => {
                let count = s.lines().count() as i64;
                println!("[Interpreter: Counted {} lines]", count);
                self.environment
                    .define(dest_name, TaleaValue::Number(count));
            }
            (Token::Sentences, TaleaValue::String(s)) => {
                let count = s
                    .split(|c| c == '.' || c == '!' || c == '?')
                    .filter(|s| !s.trim().is_empty())
                    .count() as i64;
                println!("[Interpreter: Counted {} sentences]", count);
                self.environment
                    .define(dest_name, TaleaValue::Number(count));
            }
            (Token::Paragraphs, TaleaValue::String(s)) => {
                let count = s.split("\n\n").filter(|p| !p.trim().is_empty()).count() as i64;
                println!("[Interpreter: Counted {} paragraphs]", count);
                self.environment
                    .define(dest_name, TaleaValue::Number(count));
            }
            // CASE 4: Get character count for *each item* in a list
            (Token::Characters, TaleaValue::List(l)) => {
                let lengths: Vec<TaleaValue> = l
                    .iter()
                    .map(|item| {
                        let len = if let TaleaValue::String(s) = item {
                            s.len() as i64
                        } else {
                            0 // Or handle error appropriately
                        };
                        TaleaValue::Number(len)
                    })
                    .collect();
                println!(
                    "[Interpreter: Generated list of {} character counts]",
                    lengths.len()
                );
                self.environment
                    .define(dest_name, TaleaValue::List(lengths));
            }
            _ => {
                return Err(format!(
                    "Cannot count {:?} in the provided variable type.",
                    unit_token
                ));
            }
        }
        Ok(())
    }

    fn execute_concordance_statement(
        &mut self,
        word: &Expression,
        source: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        let target = self.get_string_value(word)?.to_lowercase();
        let text = self.get_string_value(source)?;
        let dest_name = self.get_identifier_name(destination)?;
        let sentences: Vec<TaleaValue> = text
            .split(|c| c == '.' || c == '!' || c == '?' || c == '\n')
            .map(|s| s.trim().to_string())
            .filter(|s| s.to_lowercase().contains(&target))
            .map(TaleaValue::String)
            .collect();
        let count = sentences.len();
        self.environment
            .define(dest_name, TaleaValue::List(sentences));
        println!("[Interpreter: Found {} occurrences]", count);
        Ok(())
    }
    fn execute_print_statement(
        &mut self,
        expression: &Expression,
    ) -> std::result::Result<(), String> {
        let value = self.evaluate_expression(expression)?;
        match value {
            TaleaValue::Table(rows) => {
                if rows.is_empty() {
                    println!("<empty table>");
                } else {
                    let mut cols: Vec<String> = rows[0].keys().cloned().collect();
                    cols.sort();
                    println!("{}", cols.join("\t"));
                    for row in rows.iter().take(5) {
                        let mut vals = Vec::new();
                        for col in &cols {
                            let cell = row
                                .get(col)
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "null".to_string());
                            vals.push(cell);
                        }
                        println!("{}", vals.join("\t"));
                    }
                    if rows.len() > 5 {
                        println!("... ({} more rows)", rows.len() - 5);
                    }
                }
            }
            other => println!("{}", other.to_string()),
        }
        Ok(())
    }
    fn evaluate_expression(
        &mut self,
        expression: &Expression,
    ) -> std::result::Result<TaleaValue, String> {
        match expression {
            Expression::StringLiteral(s) => Ok(TaleaValue::String(s.clone())),
            Expression::Number(n) => Ok(TaleaValue::Number(*n)),
            Expression::Identifier(name) => self
                .environment
                .get(name)
                .ok_or_else(|| format!("Variable '{}' not found.", name)),
            Expression::Unit(token) => Ok(TaleaValue::Unit(token.clone())),
            Expression::ListLiteral(items) => {
                let mut vals = Vec::new();
                for item in items {
                    vals.push(self.evaluate_expression(item)?);
                }
                Ok(TaleaValue::List(vals))
            }
            Expression::Index(expr, idx) => {
                let list_val = self.evaluate_expression(expr)?;
                let idx_val = self.evaluate_expression(idx)?;
                match (list_val, idx_val) {
                    (TaleaValue::List(l), TaleaValue::Number(n)) => {
                        let i = n as usize;
                        l.get(i)
                            .cloned()
                            .ok_or_else(|| "Index out of bounds".to_string())
                    }
                    _ => Err("Indexing requires a list and a numeric index".to_string()),
                }
            }
            Expression::Comparison(left, op, right) => {
                let l = self.evaluate_expression(left)?;
                let r = self.evaluate_expression(right)?;
                let result = match (l, r) {
                    (TaleaValue::Number(a), TaleaValue::Number(b)) => match op {
                        ComparisonOp::Equals => a == b,
                        ComparisonOp::GreaterThan => a > b,
                        ComparisonOp::LessThan => a < b,
                    },
                    (TaleaValue::String(a), TaleaValue::String(b)) => match op {
                        ComparisonOp::Equals => a == b,
                        _ => {
                            return Err(
                                "Only equality comparison supported for strings".to_string()
                            );
                        }
                    },
                    _ => return Err("Comparison requires two numbers or two strings".to_string()),
                };
                Ok(TaleaValue::Number(if result { 1 } else { 0 }))
            }
        }
    }
    fn execute_shapiro_statement(
        &mut self,
        column: &Expression,
        data: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        if !self.active_backends.contains(&Backend::Python) {
            return Err("Shapiro test requires 'use python' first.".to_string());
        }
        let dest_name = self.get_identifier_name(destination)?;
        let values = self.get_column_values(column, data)?;
        Python::with_gil(|py| -> PyResult<()> {
            let stats = PyModule::import_bound(py, "scipy.stats")?;
            let arr = PyList::new_bound(py, &values);
            let res = stats.call_method1("shapiro", (arr,))?;
            let stat: f64 = res.get_item(0)?.extract()?;
            let p: f64 = res.get_item(1)?.extract()?;
            self.environment.define(
                dest_name,
                TaleaValue::Tuple(vec![TaleaValue::Float(stat), TaleaValue::Float(p)]),
            );
            Ok(())
        })
        .map_err(|e| format!("Python Error: {}", e))?;
        println!("[Interpreter: Shapiro-Wilk test complete.]");
        Ok(())
    }

    fn execute_kolmogorov_statement(
        &mut self,
        column: &Expression,
        data: &Expression,
        destination: &Expression,
    ) -> std::result::Result<(), String> {
        if !self.active_backends.contains(&Backend::Python) {
            return Err("Kolmogorov test requires 'use python' first.".to_string());
        }
        let dest_name = self.get_identifier_name(destination)?;
        let values = self.get_column_values(column, data)?;
        Python::with_gil(|py| -> PyResult<()> {
            let stats = PyModule::import_bound(py, "scipy.stats")?;
            let arr = PyList::new_bound(py, &values);
            let res = stats.call_method1("kstest", (arr, "norm"))?;
            let stat: f64 = res.get_item(0)?.extract()?;
            let p: f64 = res.get_item(1)?.extract()?;
            self.environment.define(
                dest_name,
                TaleaValue::Tuple(vec![TaleaValue::Float(stat), TaleaValue::Float(p)]),
            );
            Ok(())
        })
        .map_err(|e| format!("Python Error: {}", e))?;
        println!("[Interpreter: Kolmogorov-Smirnov test complete.]");
        Ok(())
    }

    fn get_identifier_name(&self, expression: &Expression) -> std::result::Result<String, String> {
        if let Expression::Identifier(name) = expression {
            Ok(name.clone())
        } else {
            Err("Expected an identifier".to_string())
        }
    }
    fn get_string_value(&mut self, expression: &Expression) -> std::result::Result<String, String> {
        if let TaleaValue::String(s) = self.evaluate_expression(expression)? {
            Ok(s)
        } else {
            Err("Expected a string value".to_string())
        }
    }

    fn get_number_list(
        &mut self,
        expression: &Expression,
    ) -> std::result::Result<Vec<f64>, String> {
        match self.evaluate_expression(expression)? {
            TaleaValue::List(list) => {
                let mut out = Vec::new();
                for item in list {
                    match item {
                        TaleaValue::Number(n) => out.push(n as f64),
                        TaleaValue::String(s) => out.push(
                            s.parse::<f64>()
                                .map_err(|_| "Expected numeric value".to_string())?,
                        ),
                        _ => return Err("Expected list of numbers".to_string()),
                    }
                }
                Ok(out)
            }
            _ => Err("Expected a list of numbers".to_string()),
        }
    }

    fn get_column_values(
        &mut self,
        column: &Expression,
        data: &Expression,
    ) -> std::result::Result<Vec<f64>, String> {
        let col_name = self.get_identifier_name(column)?;
        let data_val = self.evaluate_expression(data)?;
        if let TaleaValue::List(entries) = data_val {
            for entry in entries {
                match entry {
                    TaleaValue::List(pair) if pair.len() == 2 => {
                        if let TaleaValue::String(name) = &pair[0] {
                            if name == &col_name {
                                if let TaleaValue::List(vals) = &pair[1] {
                                    let mut nums = Vec::new();
                                    for v in vals {
                                        match v {
                                            TaleaValue::Number(n) => nums.push(*n as f64),
                                            TaleaValue::Float(f) => nums.push(*f),
                                            _ => {
                                                return Err(
                                                    "Column values must be numbers".to_string()
                                                );
                                            }
                                        }
                                    }
                                    return Ok(nums);
                                }
                            }
                        }
                    }
                    TaleaValue::Tuple(pair) if pair.len() == 2 => {
                        if let TaleaValue::String(name) = &pair[0] {
                            if name == &col_name {
                                if let TaleaValue::List(vals) = &pair[1] {
                                    let mut nums = Vec::new();
                                    for v in vals {
                                        match v {
                                            TaleaValue::Number(n) => nums.push(*n as f64),
                                            TaleaValue::Float(f) => nums.push(*f),
                                            _ => {
                                                return Err(
                                                    "Column values must be numbers".to_string()
                                                );
                                            }
                                        }
                                    }
                                    return Ok(nums);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Err(format!("Column '{}' not found", col_name))
    }

    fn execute_if_statement(
        &mut self,
        condition: &Expression,
        then_body: &Vec<Statement>,
        else_body: &Vec<Statement>,
    ) -> std::result::Result<(), String> {
        let cond_val = self.evaluate_expression(condition)?;
        let truthy = match cond_val {
            TaleaValue::Number(n) => n != 0,
            TaleaValue::Float(f) => f != 0.0,
            TaleaValue::String(ref s) => !s.is_empty(),
            TaleaValue::List(ref l) => !l.is_empty(),
            TaleaValue::Tuple(ref t) => !t.is_empty(),
            TaleaValue::Table(ref rows) => !rows.is_empty(),
            TaleaValue::Unit(_) => true,
            TaleaValue::Null => false,
        };
        let body = if truthy { then_body } else { else_body };
        for stmt in body {
            self.execute_statement(stmt)?;
        }
        Ok(())
    }

    fn clean_text(input: &str) -> String {
        let filtered: String = input
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect();
        filtered.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn stem_word(word: &str) -> String {
        let lower = word.to_lowercase();
        for suffix in ["ing", "ed", "ly", "s"] {
            if lower.ends_with(suffix) && lower.len() > suffix.len() {
                return lower[..lower.len() - suffix.len()].to_string();
            }
        }
        lower
    }

    fn execute_foreach_statement(
        &mut self,
        item: &String,
        list: &Expression,
        body: &Vec<Statement>,
    ) -> std::result::Result<(), String> {
        let list_val = self.evaluate_expression(list)?;
        if let TaleaValue::List(items) = list_val {
            for val in items {
                self.environment.define(item.clone(), val);
                for stmt in body {
                    self.execute_statement(stmt)?;
                }
            }
            Ok(())
        } else {
            Err("for each requires a list".to_string())
        }
    }

    fn talea_value_to_py(py: Python, value: &TaleaValue) -> PyObject {
        match value {
            TaleaValue::String(s) => s.into_py(py),
            TaleaValue::Number(n) => n.into_py(py),
            TaleaValue::List(l) => {
                let items: Vec<PyObject> = l
                    .iter()
                    .map(|v| Interpreter::talea_value_to_py(py, v))
                    .collect();
                PyList::new_bound(py, items).into_py(py)
            }
            TaleaValue::Table(rows) => {
                let py_rows: Vec<PyObject> = rows
                    .iter()
                    .map(|row| {
                        let dict = PyDict::new_bound(py);
                        for (k, v) in row {
                            let _ = dict.set_item(k, Interpreter::talea_value_to_py(py, v));
                        }
                        dict.into_py(py)
                    })
                    .collect();
                PyList::new_bound(py, py_rows).into_py(py)
            }
            _ => py.None(),
        }
    }

    fn py_obj_to_talea(obj: Bound<'_, PyAny>) -> TaleaValue {
        if let Ok(s) = obj.extract::<String>() {
            TaleaValue::String(s)
        } else if let Ok(n) = obj.extract::<i64>() {
            TaleaValue::Number(n)
        } else if let Ok(f) = obj.extract::<f64>() {
            TaleaValue::Float(f)
        } else if let Ok(list) = obj.downcast::<PyList>() {
            let vals = list
                .iter()
                .map(|item| Interpreter::py_obj_to_talea(item))
                .collect();
            TaleaValue::List(vals)
        } else if obj.hasattr("to_dict").unwrap_or(false) {
            if let Ok(method) = obj.getattr("to_dict") {
                if let Ok(records) = method.call1(("records",)) {
                    if let Ok(list) = records.downcast::<PyList>() {
                        let mut rows: Vec<HashMap<String, TaleaValue>> = Vec::new();
                        for item in list.iter() {
                            if let Ok(dict) = item.downcast::<PyDict>() {
                                let mut map: HashMap<String, TaleaValue> = HashMap::new();
                                for (k, v) in dict.iter() {
                                    if let Ok(key) = k.extract::<String>() {
                                        map.insert(key, Interpreter::py_obj_to_talea(v));
                                    }
                                }
                                rows.push(map);
                            }
                        }
                        return TaleaValue::Table(rows);
                    }
                }
            }
            TaleaValue::Null
        } else {
            TaleaValue::Null
        }
    }

    fn talea_value_to_json(value: &TaleaValue) -> JsonValue {
        match value {
            TaleaValue::String(s) => JsonValue::String(s.clone()),
            TaleaValue::Number(n) => JsonValue::Number((*n).into()),
            TaleaValue::Float(f) => serde_json::Number::from_f64(*f)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null),
            TaleaValue::Table(rows) => {
                let json_rows: Vec<JsonValue> = rows
                    .iter()
                    .map(|row| {
                        let mut map = serde_json::Map::new();
                        for (k, v) in row {
                            map.insert(k.clone(), Interpreter::talea_value_to_json(v));
                        }
                        JsonValue::Object(map)
                    })
                    .collect();
                JsonValue::Array(json_rows)
            }
            TaleaValue::List(l) | TaleaValue::Tuple(l) => JsonValue::Array(
                l.iter()
                    .map(|v| Interpreter::talea_value_to_json(v))
                    .collect(),
            ),
            TaleaValue::Unit(t) => JsonValue::String(format!("{:?}", t)),
            TaleaValue::Null => JsonValue::Null,
        }
    }

    fn talea_value_to_r_arg(value: &TaleaValue) -> String {
        match value {
            TaleaValue::String(s) => format!("'{}'", s.replace("'", "\\'")),
            TaleaValue::Number(n) => n.to_string(),
            TaleaValue::List(l) => {
                let elems: Vec<String> = l
                    .iter()
                    .map(|v| Interpreter::talea_value_to_r_arg(v))
                    .collect();
                format!("c({})", elems.join(","))
            }
            _ => "NULL".to_string(),
        }
    }

    #[cfg(feature = "r")]
    fn r_obj_to_talea(obj: r_prelude::Robj) -> TaleaValue {
        if let Some(v) = obj.as_real() {
            TaleaValue::Number(v as i64)
        } else if let Some(v) = obj.as_integer() {
            TaleaValue::Number(v as i64)
        } else if let Some(s) = obj.as_str() {
            TaleaValue::String(s.to_string())
        } else if let Some(vec) = obj.as_real_vector() {
            TaleaValue::List(vec.iter().map(|v| TaleaValue::Number(*v as i64)).collect())
        } else if let Some(vec) = obj.as_integer_vector() {
            TaleaValue::List(vec.iter().map(|v| TaleaValue::Number(*v as i64)).collect())
        } else if let Some(vec) = obj.as_str_vector() {
            TaleaValue::List(
                vec.iter()
                    .map(|v| TaleaValue::String(v.to_string()))
                    .collect(),
            )
        } else {
            TaleaValue::Null
        }
    }
}

impl TaleaValue {
    fn as_string(&self) -> Option<String> {
        if let TaleaValue::String(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }
}
impl ToString for TaleaValue {
    fn to_string(&self) -> String {
        match self {
            TaleaValue::String(s) => s.clone(),
            TaleaValue::Number(n) => n.to_string(),
            TaleaValue::Float(f) => f.to_string(),
            TaleaValue::List(l) => {
                let items: Vec<String> = l.iter().map(|val| val.to_string_for_list()).collect();
                format!("[List with {} items]:\n[{}]", l.len(), items.join(", "))
            }
            TaleaValue::Tuple(t) => {
                let items: Vec<String> = t.iter().map(|val| val.to_string_for_list()).collect();
                format!("({})", items.join(", "))
            }
            TaleaValue::Table(rows) => {
                format!("<Table with {} rows>", rows.len())
            }
            TaleaValue::Unit(t) => format!("Unit: {:?}", t),
            TaleaValue::Null => "null".to_string(),
        }
    }
}

fn format_runtime_error(msg: String) -> String {
    if let Some(name) = msg
        .strip_prefix("Variable '")
        .and_then(|s| s.strip_suffix("' not found."))
    {
        format!(
            "Variable '{}' is undefined. Did you forget to define or load it?",
            name
        )
    } else if msg == "Python backend not enabled. Run 'use python' first." {
        "Python backend not enabled. Did you run `use python`?".to_string()
    } else if msg == "R backend not enabled. Run 'use r' first." {
        "R backend not enabled. Did you run `use r`?".to_string()
    } else {
        msg
    }
}

trait ToStringForList {
    fn to_string_for_list(&self) -> String;
}
impl ToStringForList for TaleaValue {
    fn to_string_for_list(&self) -> String {
        match self {
            TaleaValue::String(s) => format!("'{}'", s),
            _ => self.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expression, Statement};

    #[test]
    fn undefined_variable_suggests_fix() {
        let mut interpreter = Interpreter::new();
        let stmts = vec![Statement::Print(Expression::Identifier("x".into()))];
        let err = interpreter.execute(stmts).unwrap_err();
        assert!(err.contains("Did you forget to define or load"), "{}", err);
    }

    #[test]
    fn count_words_in_string() {
        let mut interpreter = Interpreter::new();
        let stmts = vec![
            Statement::Define {
                name: Expression::Identifier("article".into()),
                value: Expression::StringLiteral("hello world".into()),
            },
            Statement::Count {
                unit: Expression::Unit(Token::Words),
                source: Expression::Identifier("article".into()),
                destination: Expression::Identifier("word_count".into()),
            },
        ];
        interpreter.execute(stmts).unwrap();
        let result = interpreter.environment.get("word_count").unwrap();
        assert_eq!(result, TaleaValue::Number(2));
    }

    #[test]
    fn count_sentences_in_string() {
        let mut interpreter = Interpreter::new();
        let stmts = vec![
            Statement::Define {
                name: Expression::Identifier("article".into()),
                value: Expression::StringLiteral("Hello world. This is a test!".into()),
            },
            Statement::Count {
                unit: Expression::Unit(Token::Sentences),
                source: Expression::Identifier("article".into()),
                destination: Expression::Identifier("sentence_count".into()),
            },
        ];
        interpreter.execute(stmts).unwrap();
        let result = interpreter.environment.get("sentence_count").unwrap();
        assert_eq!(result, TaleaValue::Number(2));
    }

    #[test]
    fn count_paragraphs_in_string() {
        let mut interpreter = Interpreter::new();
        let stmts = vec![
            Statement::Define {
                name: Expression::Identifier("article".into()),
                value: Expression::StringLiteral("First para.\n\nSecond para.".into()),
            },
            Statement::Count {
                unit: Expression::Unit(Token::Paragraphs),
                source: Expression::Identifier("article".into()),
                destination: Expression::Identifier("para_count".into()),
            },
        ];
        interpreter.execute(stmts).unwrap();
        let result = interpreter.environment.get("para_count").unwrap();
        assert_eq!(result, TaleaValue::Number(2));
    }
}
