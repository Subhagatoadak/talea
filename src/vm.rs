// src/vm.rs
//
// Stack-based virtual machine that executes Talea bytecode (`Chunk`).

use std::collections::{HashMap, HashSet};

use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::ast::Backend;
use crate::bytecode::{Chunk, Constant, OpCode};
use crate::openai_pipeline::OpenAiPipeline;
use crate::runtime::{Environment, TaleaValue};

// ──────────────────────────────────────────────────────────────
// VM
// ──────────────────────────────────────────────────────────────
pub struct VM {
    stack: Vec<TaleaValue>,
    env: Environment,
    active_backends: HashSet<Backend>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: Vec::new(),
            env: Environment::new(),
            active_backends: HashSet::new(),
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<TaleaValue> {
        self.env.get(name)
    }

    // ─── Main execution loop ────────────────────────────────────────────────

    pub fn execute(&mut self, chunk: &Chunk) -> Result<(), String> {
        let mut ip = 0usize;
        let instructions = &chunk.instructions;
        let constants = &chunk.constants;

        while ip < instructions.len() {
            let op = &instructions[ip];
            ip += 1;

            match op {
                // ── Stack / variables ─────────────────────────────────────
                OpCode::LoadConst(idx) => {
                    let val = constant_to_value(constants.get(*idx).ok_or_else(|| {
                        format!("Constant index {} out of range", idx)
                    })?);
                    self.push(val);
                }
                OpCode::LoadVar(name) => {
                    let val = self.env.get(name).ok_or_else(|| {
                        format!("Undefined variable '{}'", name)
                    })?;
                    self.push(val);
                }
                OpCode::StoreVar(name) => {
                    let val = self.pop()?;
                    self.env.define(name.clone(), val);
                }
                OpCode::Index => {
                    let index = self.pop()?;
                    let collection = self.pop()?;
                    let result = vm_index(collection, index)?;
                    self.push(result);
                }
                OpCode::BuildList(n) => {
                    let mut items = Vec::with_capacity(*n);
                    for _ in 0..*n {
                        items.push(self.pop()?);
                    }
                    items.reverse();
                    self.push(TaleaValue::List(items));
                }
                OpCode::Pop => {
                    self.pop()?;
                }
                OpCode::Nop | OpCode::Label(_) => {}

                // ── I/O ───────────────────────────────────────────────────
                OpCode::Print => {
                    let val = self.pop()?;
                    println!("{}", display_value(&val));
                }
                OpCode::Load => {
                    let alias_val = self.pop()?;
                    let path_val = self.pop()?;
                    let path = require_string(path_val, "load")?;
                    let alias = require_string(alias_val, "load alias")?;
                    let expanded = shellexpand::tilde(&path).into_owned();
                    let content = std::fs::read_to_string(&expanded)
                        .map_err(|e| format!("Cannot read '{}': {}", path, e))?;
                    self.env.define(alias, TaleaValue::String(content));
                }
                OpCode::LoadCsv => {
                    let alias_val = self.pop()?;
                    let path_val = self.pop()?;
                    let path = require_string(path_val, "load csv")?;
                    let alias = require_string(alias_val, "load csv alias")?;
                    let expanded = shellexpand::tilde(&path).into_owned();
                    let content = std::fs::read_to_string(&expanded)
                        .map_err(|e| format!("Cannot read CSV '{}': {}", path, e))?;
                    self.env.define(alias, TaleaValue::String(content));
                }
                OpCode::Save => {
                    let dest_val = self.pop()?;
                    let src_val = self.pop()?;
                    let dest = require_string(dest_val, "save destination")?;
                    let src = require_string(src_val, "save source")?;
                    std::fs::write(&dest, src)
                        .map_err(|e| format!("Cannot write '{}': {}", dest, e))?;
                }
                OpCode::Define => {
                    let name_val = self.pop()?;
                    let value = self.pop()?;
                    let name = require_string(name_val, "define name")?;
                    self.env.define(name, value);
                }

                // ── NLP ───────────────────────────────────────────────────
                OpCode::Tokenize => {
                    let src = self.pop()?;
                    let text = require_string(src, "tokenize")?;
                    let tokens: Vec<TaleaValue> = text
                        .split_whitespace()
                        .map(|w| TaleaValue::String(w.to_string()))
                        .collect();
                    self.push(TaleaValue::List(tokens));
                }
                OpCode::Sentiment => {
                    let src = self.pop()?;
                    let text = require_string(src, "sentiment")?.to_lowercase();
                    let positive = ["good","great","happy","love","excellent","awesome","fantastic","nice"];
                    let negative = ["bad","terrible","sad","hate","awful","horrible","poor","worst"];
                    let mut score = 0i32;
                    for word in text.split_whitespace() {
                        if positive.contains(&word) { score += 1; }
                        if negative.contains(&word) { score -= 1; }
                    }
                    let s = if score > 0 { "positive" } else if score < 0 { "negative" } else { "neutral" };
                    self.push(TaleaValue::String(s.to_string()));
                }
                OpCode::Count => {
                    let src = self.pop()?;
                    let unit_val = self.pop()?;
                    let unit = require_string(unit_val, "count unit")?.to_lowercase();
                    let count = match unit.as_str() {
                        "words" | "word" => match &src {
                            TaleaValue::String(s) => s.split_whitespace().count() as i64,
                            TaleaValue::List(l) => l.len() as i64,
                            _ => 0,
                        },
                        _ => match &src {
                            TaleaValue::List(l) => l.len() as i64,
                            TaleaValue::String(s) => s.len() as i64,
                            _ => 0,
                        },
                    };
                    self.push(TaleaValue::Number(count));
                }
                OpCode::Split => {
                    let src = self.pop()?;
                    let unit_val = self.pop()?;
                    let unit = require_string(unit_val, "split unit")?.to_lowercase();
                    let text = require_string(src, "split source")?;
                    let parts: Vec<TaleaValue> = match unit.as_str() {
                        "sentences" | "sentence" => text
                            .split(|c| c == '.' || c == '!' || c == '?')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .map(TaleaValue::String)
                            .collect(),
                        _ => text
                            .split_whitespace()
                            .map(|w| TaleaValue::String(w.to_string()))
                            .collect(),
                    };
                    self.push(TaleaValue::List(parts));
                }
                OpCode::Join => {
                    let sep = self.pop()?;
                    let src = self.pop()?;
                    let _unit = self.pop()?;
                    let separator = require_string(sep, "join separator")?;
                    let items = match src {
                        TaleaValue::List(l) => l
                            .into_iter()
                            .map(|v| match v {
                                TaleaValue::String(s) => s,
                                other => display_value(&other),
                            })
                            .collect::<Vec<_>>(),
                        other => vec![display_value(&other)],
                    };
                    self.push(TaleaValue::String(items.join(&separator)));
                }
                OpCode::Replace => {
                    let src = self.pop()?;
                    let replacement = self.pop()?;
                    let pattern = self.pop()?;
                    let text = require_string(src, "replace source")?;
                    let pat = require_string(pattern, "replace pattern")?;
                    let rep = require_string(replacement, "replace replacement")?;
                    self.push(TaleaValue::String(text.replace(&pat, &rep)));
                }
                OpCode::Clean => {
                    let src = self.pop()?;
                    let text = require_string(src, "clean")?;
                    let cleaned: String = text.chars().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect();
                    self.push(TaleaValue::String(cleaned));
                }
                OpCode::Lowercase => {
                    let src = self.pop()?;
                    let text = require_string(src, "lowercase")?;
                    self.push(TaleaValue::String(text.to_lowercase()));
                }
                OpCode::Uppercase => {
                    let src = self.pop()?;
                    let text = require_string(src, "uppercase")?;
                    self.push(TaleaValue::String(text.to_uppercase()));
                }
                OpCode::Stem => {
                    let src = self.pop()?;
                    let text = require_string(src, "stem")?;
                    // Simple suffix-stripping stemmer
                    let stemmed: Vec<String> = text
                        .split_whitespace()
                        .map(|w| simple_stem(w))
                        .collect();
                    self.push(TaleaValue::List(
                        stemmed.into_iter().map(TaleaValue::String).collect(),
                    ));
                }
                OpCode::Normalize => {
                    let src = self.pop()?;
                    let text = require_string(src, "normalize")?;
                    let normalized = text
                        .split_whitespace()
                        .map(|w| w.to_lowercase())
                        .collect::<Vec<_>>()
                        .join(" ");
                    self.push(TaleaValue::String(normalized));
                }
                OpCode::Lemmatize => {
                    if !self.active_backends.contains(&Backend::Python) {
                        return Err("Lemmatize requires 'use python'".to_string());
                    }
                    let src = self.pop()?;
                    let text = require_string(src, "lemmatize")?;
                    let result = Python::with_gil(|py| -> PyResult<Vec<String>> {
                        let spacy = PyModule::import_bound(py, "spacy")?;
                        let nlp = spacy.call_method1("load", ("en_core_web_sm",))?;
                        let doc = nlp.call1((&text,))?;
                        doc.iter()?
                            .map(|tok| {
                                tok?.getattr("lemma_")?.extract::<String>()
                            })
                            .collect()
                    })
                    .map_err(|e| format!("Python error: {}", e))?;
                    self.push(TaleaValue::List(
                        result.into_iter().map(TaleaValue::String).collect(),
                    ));
                }
                OpCode::Tag => {
                    if !self.active_backends.contains(&Backend::Python) {
                        return Err("Tag requires 'use python'".to_string());
                    }
                    let method_val = self.pop()?;
                    let src = self.pop()?;
                    let text = require_string(src, "tag")?;
                    let method = require_string(method_val, "tag method")?.to_lowercase();
                    let result = Python::with_gil(|py| -> PyResult<Vec<TaleaValue>> {
                        let spacy = PyModule::import_bound(py, "spacy")?;
                        let nlp = spacy.call_method1("load", ("en_core_web_md",))?;
                        let doc = nlp.call1((&text,))?;
                        match method.as_str() {
                            "ner" => doc
                                .getattr("ents")?
                                .iter()?
                                .map(|ent| {
                                    let e = ent?;
                                    let t = e.getattr("text")?.extract::<String>()?;
                                    let l = e.getattr("label_")?.extract::<String>()?;
                                    Ok(TaleaValue::Tuple(vec![
                                        TaleaValue::String(t),
                                        TaleaValue::String(l),
                                    ]))
                                })
                                .collect(),
                            _ => doc
                                .iter()?
                                .map(|tok| {
                                    let t = tok?;
                                    let text = t.getattr("text")?.extract::<String>()?;
                                    let pos = t.getattr("pos_")?.extract::<String>()?;
                                    Ok(TaleaValue::Tuple(vec![
                                        TaleaValue::String(text),
                                        TaleaValue::String(pos),
                                    ]))
                                })
                                .collect(),
                        }
                    })
                    .map_err(|e| format!("Python error: {}", e))?;
                    self.push(TaleaValue::List(result));
                }
                OpCode::Concordance => {
                    let src = self.pop()?;
                    let word_val = self.pop()?;
                    let word = require_string(word_val, "concordance word")?.to_lowercase();
                    let text = require_string(src, "concordance source")?;
                    let results: Vec<TaleaValue> = text
                        .split(|c| c == '.' || c == '!' || c == '?')
                        .filter(|s| s.to_lowercase().contains(&word))
                        .map(|s| TaleaValue::String(s.trim().to_string()))
                        .collect();
                    self.push(TaleaValue::List(results));
                }
                OpCode::WordCloud => {
                    let src = self.pop()?;
                    let tokens = collect_tokens(src)?;
                    let mut counts: HashMap<String, i64> = HashMap::new();
                    for t in tokens {
                        *counts.entry(t).or_insert(0) += 1;
                    }
                    let mut pairs: Vec<(String, i64)> = counts.into_iter().collect();
                    pairs.sort_by(|a, b| b.1.cmp(&a.1));
                    let result: Vec<TaleaValue> = pairs
                        .into_iter()
                        .map(|(w, c)| TaleaValue::Tuple(vec![TaleaValue::String(w), TaleaValue::Number(c)]))
                        .collect();
                    self.push(TaleaValue::List(result));
                }
                OpCode::Association => {
                    let src = self.pop()?;
                    let tokens = collect_tokens(src)?;
                    let mut counts: HashMap<(String, String), i64> = HashMap::new();
                    for pair in tokens.windows(2) {
                        if let [a, b] = pair {
                            *counts.entry((a.clone(), b.clone())).or_insert(0) += 1;
                        }
                    }
                    let mut pairs: Vec<((String, String), i64)> = counts.into_iter().collect();
                    pairs.sort_by(|a, b| b.1.cmp(&a.1));
                    let result: Vec<TaleaValue> = pairs
                        .into_iter()
                        .map(|((a, b), c)| {
                            TaleaValue::Tuple(vec![
                                TaleaValue::String(a),
                                TaleaValue::String(b),
                                TaleaValue::Number(c),
                            ])
                        })
                        .collect();
                    self.push(TaleaValue::List(result));
                }
                OpCode::SemanticSimilarity => {
                    let s2_val = self.pop()?;
                    let s1_val = self.pop()?;
                    let s1 = require_string(s1_val, "similarity text1")?;
                    let s2 = require_string(s2_val, "similarity text2")?;
                    let set1: HashSet<String> = s1.split_whitespace().map(|w| w.to_lowercase()).collect();
                    let set2: HashSet<String> = s2.split_whitespace().map(|w| w.to_lowercase()).collect();
                    let inter = set1.intersection(&set2).count() as i64;
                    let union = set1.union(&set2).count().max(1) as i64;
                    self.push(TaleaValue::Number(inter * 100 / union));
                }
                OpCode::SemanticSearch => {
                    let src = self.pop()?;
                    let query_val = self.pop()?;
                    let query = require_string(query_val, "semantic search query")?;
                    let query_set: HashSet<String> = query.split_whitespace().map(|w| w.to_lowercase()).collect();
                    let docs = match src {
                        TaleaValue::List(l) => l,
                        _ => return Err("Semantic search expects a list of strings".to_string()),
                    };
                    let mut results: Vec<(i64, i64)> = docs
                        .iter()
                        .enumerate()
                        .map(|(i, doc)| {
                            let doc_set: HashSet<String> = match doc {
                                TaleaValue::String(s) => s.split_whitespace().map(|w| w.to_lowercase()).collect(),
                                _ => HashSet::new(),
                            };
                            let inter = query_set.intersection(&doc_set).count() as i64;
                            let union = query_set.union(&doc_set).count().max(1) as i64;
                            (i as i64, inter * 100 / union)
                        })
                        .collect();
                    results.sort_by(|a, b| b.1.cmp(&a.1));
                    let list: Vec<TaleaValue> = results
                        .into_iter()
                        .map(|(i, s)| TaleaValue::Tuple(vec![TaleaValue::Number(i), TaleaValue::Number(s)]))
                        .collect();
                    self.push(TaleaValue::List(list));
                }
                OpCode::KeywordSearch => {
                    let src = self.pop()?;
                    let query_val = self.pop()?;
                    let target = require_string(query_val, "keyword search")?.to_lowercase();
                    let text = require_string(src, "keyword source")?;
                    let sentences: Vec<TaleaValue> = text
                        .split(|c| c == '.' || c == '!' || c == '?' || c == '\n')
                        .map(|s| s.trim().to_string())
                        .filter(|s| s.to_lowercase().contains(&target))
                        .map(TaleaValue::String)
                        .collect();
                    self.push(TaleaValue::List(sentences));
                }
                OpCode::Translate => {
                    let lang_val = self.pop()?;
                    let src_val = self.pop()?;
                    let text = require_string(src_val, "translate")?;
                    let lang = require_string(lang_val, "translate language")?;
                    let prompt = format!("Translate the following text to {}:\n{}", lang, text);
                    let pipeline = OpenAiPipeline::from_env("gpt-3.5-turbo")?;
                    let translation = pipeline.ask(&prompt)?;
                    self.push(TaleaValue::String(translation));
                }
                OpCode::Summarize => {
                    let src = self.pop()?;
                    let summary = match src {
                        TaleaValue::String(s) => {
                            s.split(|c| c == '.' || c == '!' || c == '?')
                                .next()
                                .unwrap_or("")
                                .trim()
                                .to_string()
                        }
                        _ => return Err("Summarize expects a string".to_string()),
                    };
                    self.push(TaleaValue::String(summary));
                }
                OpCode::Doc2Bow => {
                    let src = self.pop()?;
                    let tokens = collect_tokens(src)?;
                    let mut counts: HashMap<String, i64> = HashMap::new();
                    for t in tokens {
                        *counts.entry(t).or_insert(0) += 1;
                    }
                    let pairs: Vec<TaleaValue> = counts
                        .into_iter()
                        .map(|(w, c)| TaleaValue::Tuple(vec![TaleaValue::String(w), TaleaValue::Number(c)]))
                        .collect();
                    self.push(TaleaValue::List(pairs));
                }

                // ── Statistics ────────────────────────────────────────────
                OpCode::Regress => {
                    let _with_openai_val = self.pop()?;
                    let _src = self.pop()?;
                    let _nfeatures_val = self.pop()?;
                    // For VM we push a placeholder result
                    self.push(TaleaValue::String("[Regression result]".to_string()));
                }
                OpCode::TTest => {
                    if !self.active_backends.contains(&Backend::Python) {
                        return Err("TTest requires 'use python'".to_string());
                    }
                    let g2 = self.pop()?;
                    let g1 = self.pop()?;
                    let _col = self.pop()?;
                    let nums1 = extract_number_list(g1)?;
                    let nums2 = extract_number_list(g2)?;
                    let result = Python::with_gil(|py| -> PyResult<TaleaValue> {
                        setup_py_path(py)?;
                        let stats = PyModule::import_bound(py, "scipy.stats")?;
                        let l1 = PyList::new_bound(py, &nums1);
                        let l2 = PyList::new_bound(py, &nums2);
                        let res = stats.call_method1("ttest_ind", (&l1, &l2))?;
                        let t: f64 = res.get_item(0)?.extract()?;
                        let p: f64 = res.get_item(1)?.extract()?;
                        Ok(TaleaValue::Tuple(vec![
                            TaleaValue::String(t.to_string()),
                            TaleaValue::String(p.to_string()),
                        ]))
                    })
                    .map_err(|e| format!("Python error: {}", e))?;
                    self.push(result);
                }
                OpCode::Anova => {
                    if !self.active_backends.contains(&Backend::Python) {
                        return Err("ANOVA requires 'use python'".to_string());
                    }
                    let g2 = self.pop()?;
                    let g1 = self.pop()?;
                    let _col = self.pop()?;
                    let nums1 = extract_number_list(g1)?;
                    let nums2 = extract_number_list(g2)?;
                    let result = Python::with_gil(|py| -> PyResult<TaleaValue> {
                        setup_py_path(py)?;
                        let stats = PyModule::import_bound(py, "scipy.stats")?;
                        let l1 = PyList::new_bound(py, &nums1);
                        let l2 = PyList::new_bound(py, &nums2);
                        let res = stats.call_method1("f_oneway", (&l1, &l2))?;
                        let f: f64 = res.get_item(0)?.extract()?;
                        let p: f64 = res.get_item(1)?.extract()?;
                        Ok(TaleaValue::Tuple(vec![
                            TaleaValue::String(f.to_string()),
                            TaleaValue::String(p.to_string()),
                        ]))
                    })
                    .map_err(|e| format!("Python error: {}", e))?;
                    self.push(result);
                }
                OpCode::Shapiro => {
                    let data_val = self.pop()?;
                    let _col = self.pop()?;
                    let nums = extract_number_list(data_val)?;
                    if !self.active_backends.contains(&Backend::Python) {
                        // Fallback: return null
                        self.push(TaleaValue::String("shapiro-fallback".to_string()));
                    } else {
                        let result = Python::with_gil(|py| -> PyResult<TaleaValue> {
                            setup_py_path(py)?;
                            let stats = PyModule::import_bound(py, "scipy.stats")?;
                            let l = PyList::new_bound(py, &nums);
                            let res = stats.call_method1("shapiro", (&l,))?;
                            let stat: f64 = res.get_item(0)?.extract()?;
                            let p: f64 = res.get_item(1)?.extract()?;
                            Ok(TaleaValue::Tuple(vec![
                                TaleaValue::String(stat.to_string()),
                                TaleaValue::String(p.to_string()),
                            ]))
                        })
                        .map_err(|e| format!("Python error: {}", e))?;
                        self.push(result);
                    }
                }
                OpCode::Kolmogorov => {
                    let data_val = self.pop()?;
                    let _col = self.pop()?;
                    let nums = extract_number_list(data_val)?;
                    if !self.active_backends.contains(&Backend::Python) {
                        self.push(TaleaValue::String("ks-fallback".to_string()));
                    } else {
                        let result = Python::with_gil(|py| -> PyResult<TaleaValue> {
                            setup_py_path(py)?;
                            let stats = PyModule::import_bound(py, "scipy.stats")?;
                            let l = PyList::new_bound(py, &nums);
                            let res = stats.call_method1("kstest", (&l, "norm"))?;
                            let stat: f64 = res.get_item(0)?.extract()?;
                            let p: f64 = res.get_item(1)?.extract()?;
                            Ok(TaleaValue::Tuple(vec![
                                TaleaValue::String(stat.to_string()),
                                TaleaValue::String(p.to_string()),
                            ]))
                        })
                        .map_err(|e| format!("Python error: {}", e))?;
                        self.push(result);
                    }
                }
                OpCode::Mean => {
                    let src = self.pop()?;
                    let _col = self.pop()?;
                    let nums = extract_number_list_flexible(src);
                    let mean = if nums.is_empty() {
                        0.0
                    } else {
                        nums.iter().sum::<f64>() / nums.len() as f64
                    };
                    self.push(TaleaValue::Float(mean));
                }
                OpCode::Median => {
                    let src = self.pop()?;
                    let _col = self.pop()?;
                    let mut nums = extract_number_list_flexible(src);
                    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let median = if nums.is_empty() {
                        0.0
                    } else if nums.len() % 2 == 0 {
                        (nums[nums.len() / 2 - 1] + nums[nums.len() / 2]) / 2.0
                    } else {
                        nums[nums.len() / 2]
                    };
                    self.push(TaleaValue::Float(median));
                }
                OpCode::Mode => {
                    let src = self.pop()?;
                    let _col = self.pop()?;
                    let nums = extract_number_list_flexible(src);
                    let mut counts: HashMap<String, usize> = HashMap::new();
                    for n in &nums {
                        *counts.entry(format!("{}", n)).or_insert(0) += 1;
                    }
                    let mode_str = counts
                        .into_iter()
                        .max_by_key(|(_, c)| *c)
                        .map(|(k, _)| k)
                        .unwrap_or_default();
                    self.push(TaleaValue::String(mode_str));
                }
                OpCode::Skewness => {
                    let src = self.pop()?;
                    let _col = self.pop()?;
                    let nums = extract_number_list_flexible(src);
                    let skew = compute_skewness(&nums);
                    self.push(TaleaValue::Float(skew));
                }
                OpCode::Kurtosis => {
                    let src = self.pop()?;
                    let _col = self.pop()?;
                    let nums = extract_number_list_flexible(src);
                    let kurt = compute_kurtosis(&nums);
                    self.push(TaleaValue::Float(kurt));
                }
                OpCode::Insight => {
                    let src = self.pop()?;
                    let serialized = format!("{}", display_value(&src));
                    let prompt = format!("Provide narrative insights for the following data: {}", serialized);
                    let pipeline = OpenAiPipeline::from_env("gpt-3.5-turbo")?;
                    let response = pipeline.ask(&prompt)?;
                    self.push(TaleaValue::String(response));
                }

                // ── Arithmetic ────────────────────────────────────────────
                OpCode::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(arith_op(a, b, "add")?);
                }
                OpCode::Sub => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(arith_op(a, b, "sub")?);
                }
                OpCode::Mul => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(arith_op(a, b, "mul")?);
                }
                OpCode::Div => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(arith_op(a, b, "div")?);
                }

                // ── Comparison ────────────────────────────────────────────
                OpCode::CmpEq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(TaleaValue::Number(if a == b { 1 } else { 0 }));
                }
                OpCode::CmpGt => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = cmp_values(&a, &b, "gt");
                    self.push(TaleaValue::Number(if result { 1 } else { 0 }));
                }
                OpCode::CmpLt => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = cmp_values(&a, &b, "lt");
                    self.push(TaleaValue::Number(if result { 1 } else { 0 }));
                }

                // ── Control flow ──────────────────────────────────────────
                OpCode::Jump(target) => {
                    ip = *target;
                }
                OpCode::JumpIfFalse(target) => {
                    let cond = self.pop()?;
                    if is_falsy(&cond) {
                        ip = *target;
                    }
                }

                // ── Backend / env ─────────────────────────────────────────
                OpCode::UseBackend(name) => {
                    match name.as_str() {
                        "python" => {
                            self.active_backends.insert(Backend::Python);
                            println!("[VM: Python backend enabled.]");
                        }
                        "r" => {
                            self.active_backends.insert(Backend::R);
                            println!("[VM: R backend enabled.]");
                        }
                        "java" => {
                            self.active_backends.insert(Backend::Java);
                            println!("[VM: Java backend enabled.]");
                        }
                        other => {
                            println!("[VM: Unknown backend '{}'.]", other);
                        }
                    }
                }
                OpCode::SetEnv => {
                    let value_val = self.pop()?;
                    let name_val = self.pop()?;
                    let name = require_string(name_val, "set_env name")?;
                    let value = require_string(value_val, "set_env value")?;
                    std::env::set_var(&name, &value);
                }
                OpCode::Import => {
                    let _alias = self.pop()?;
                    let _module = self.pop()?;
                    let _backend = self.pop()?;
                    // import handled at interpreter level; VM acknowledges
                    self.push(TaleaValue::Null);
                }
                OpCode::Call => {
                    let n_val = self.pop()?;
                    let n = match n_val {
                        TaleaValue::Number(n) => n as usize,
                        _ => 0,
                    };
                    let mut args = Vec::with_capacity(n);
                    for _ in 0..n {
                        args.push(self.pop()?);
                    }
                    args.reverse();
                    let func_val = self.pop()?;
                    let lib_val = self.pop()?;
                    let _func = require_string(func_val, "call function")?;
                    let _lib = require_string(lib_val, "call library")?;
                    // Simplified: return Null
                    self.push(TaleaValue::Null);
                }
                OpCode::CallMethod => {
                    let n_val = self.pop()?;
                    let n = match n_val {
                        TaleaValue::Number(n) => n as usize,
                        _ => 0,
                    };
                    let mut args = Vec::with_capacity(n);
                    for _ in 0..n {
                        args.push(self.pop()?);
                    }
                    args.reverse();
                    let _method = self.pop()?;
                    let _obj = self.pop()?;
                    self.push(TaleaValue::Null);
                }
                OpCode::OpenAI => {
                    let _file_path_val = self.pop()?;
                    let model_val = self.pop()?;
                    let prompt_val = self.pop()?;
                    let prompt = require_string(prompt_val, "openai prompt")?;
                    let model = match model_val {
                        TaleaValue::String(s) => s,
                        _ => "gpt-3.5-turbo".to_string(),
                    };
                    let pipeline = OpenAiPipeline::from_env(&model)?;
                    let response = pipeline.ask(&prompt)?;
                    self.push(TaleaValue::String(response));
                }
            }
        }
        Ok(())
    }

    // ─── Stack helpers ──────────────────────────────────────────────────────

    fn push(&mut self, val: TaleaValue) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Result<TaleaValue, String> {
        self.stack.pop().ok_or_else(|| "Stack underflow".to_string())
    }
}

// ──────────────────────────────────────────────────────────────
// Free helpers
// ──────────────────────────────────────────────────────────────

fn constant_to_value(c: &Constant) -> TaleaValue {
    match c {
        Constant::String(s) => TaleaValue::String(s.clone()),
        Constant::Number(n) => TaleaValue::Number(*n),
        Constant::Float(f) => TaleaValue::Float(*f),
        Constant::Bool(b) => TaleaValue::Number(if *b { 1 } else { 0 }),
        Constant::Null => TaleaValue::Null,
    }
}

fn require_string(val: TaleaValue, context: &str) -> Result<String, String> {
    match val {
        TaleaValue::String(s) => Ok(s),
        TaleaValue::Number(n) => Ok(n.to_string()),
        TaleaValue::Float(f) => Ok(f.to_string()),
        other => Err(format!("Expected string in '{}', got {:?}", context, other)),
    }
}

fn display_value(val: &TaleaValue) -> String {
    match val {
        TaleaValue::String(s) => s.clone(),
        TaleaValue::Number(n) => n.to_string(),
        TaleaValue::Float(f) => f.to_string(),
        TaleaValue::List(l) => {
            let items: Vec<String> = l.iter().map(display_value).collect();
            format!("[{}]", items.join(", "))
        }
        TaleaValue::Tuple(t) => {
            let items: Vec<String> = t.iter().map(display_value).collect();
            format!("({})", items.join(", "))
        }
        TaleaValue::Table(rows) => format!("<Table {} rows>", rows.len()),
        TaleaValue::Null => "null".to_string(),
        TaleaValue::PyObject(_) => "<PyObject>".to_string(),
        #[cfg(feature = "r")]
        TaleaValue::RObject(_) => "<RObject>".to_string(),
        TaleaValue::Unit(t) => format!("{:?}", t),
    }
}

fn is_falsy(val: &TaleaValue) -> bool {
    match val {
        TaleaValue::Number(0) => true,
        TaleaValue::Null => true,
        TaleaValue::String(s) if s.is_empty() => true,
        TaleaValue::List(l) if l.is_empty() => true,
        _ => false,
    }
}

fn vm_index(collection: TaleaValue, index: TaleaValue) -> Result<TaleaValue, String> {
    match (collection, index) {
        (TaleaValue::List(l), TaleaValue::Number(i)) => l
            .into_iter()
            .nth(i as usize)
            .ok_or_else(|| "Index out of bounds".to_string()),
        (TaleaValue::String(s), TaleaValue::Number(i)) => {
            let ch = s.chars().nth(i as usize).ok_or_else(|| "Index out of bounds".to_string())?;
            Ok(TaleaValue::String(ch.to_string()))
        }
        _ => Err("Invalid index operation".to_string()),
    }
}

fn arith_op(a: TaleaValue, b: TaleaValue, op: &str) -> Result<TaleaValue, String> {
    match (a, b) {
        (TaleaValue::Number(x), TaleaValue::Number(y)) => Ok(TaleaValue::Number(match op {
            "add" => x + y,
            "sub" => x - y,
            "mul" => x * y,
            "div" => {
                if y == 0 { return Err("Division by zero".to_string()); }
                x / y
            }
            _ => unreachable!(),
        })),
        (TaleaValue::Float(x), TaleaValue::Float(y)) => Ok(TaleaValue::Float(match op {
            "add" => x + y,
            "sub" => x - y,
            "mul" => x * y,
            "div" => x / y,
            _ => unreachable!(),
        })),
        (TaleaValue::String(a), TaleaValue::String(b)) if op == "add" => {
            Ok(TaleaValue::String(a + &b))
        }
        _ => Err(format!("Cannot apply arithmetic op '{}' to these types", op)),
    }
}

fn cmp_values(a: &TaleaValue, b: &TaleaValue, op: &str) -> bool {
    match (a, b) {
        (TaleaValue::Number(x), TaleaValue::Number(y)) => match op {
            "gt" => x > y,
            "lt" => x < y,
            _ => x == y,
        },
        (TaleaValue::Float(x), TaleaValue::Float(y)) => match op {
            "gt" => x > y,
            "lt" => x < y,
            _ => (x - y).abs() < f64::EPSILON,
        },
        (TaleaValue::String(x), TaleaValue::String(y)) => match op {
            "gt" => x > y,
            "lt" => x < y,
            _ => x == y,
        },
        _ => false,
    }
}

fn collect_tokens(val: TaleaValue) -> Result<Vec<String>, String> {
    match val {
        TaleaValue::String(s) => Ok(s.split_whitespace().map(|w| w.to_lowercase()).collect()),
        TaleaValue::List(l) => Ok(l
            .into_iter()
            .filter_map(|v| if let TaleaValue::String(s) = v { Some(s.to_lowercase()) } else { None })
            .collect()),
        _ => Err("Expected text or token list".to_string()),
    }
}

fn extract_number_list(val: TaleaValue) -> Result<Vec<f64>, String> {
    match val {
        TaleaValue::List(l) => l
            .into_iter()
            .map(|v| match v {
                TaleaValue::Number(n) => Ok(n as f64),
                TaleaValue::Float(f) => Ok(f),
                _ => Err("Expected number in list".to_string()),
            })
            .collect(),
        _ => Err("Expected a list of numbers".to_string()),
    }
}

fn extract_number_list_flexible(val: TaleaValue) -> Vec<f64> {
    match val {
        TaleaValue::List(l) => l
            .into_iter()
            .filter_map(|v| match v {
                TaleaValue::Number(n) => Some(n as f64),
                TaleaValue::Float(f) => Some(f),
                _ => None,
            })
            .collect(),
        TaleaValue::Number(n) => vec![n as f64],
        TaleaValue::Float(f) => vec![f],
        _ => vec![],
    }
}

fn compute_skewness(nums: &[f64]) -> f64 {
    if nums.len() < 2 {
        return 0.0;
    }
    let n = nums.len() as f64;
    let mean = nums.iter().sum::<f64>() / n;
    let variance = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();
    if std_dev == 0.0 {
        return 0.0;
    }
    nums.iter().map(|x| ((x - mean) / std_dev).powi(3)).sum::<f64>() / n
}

fn compute_kurtosis(nums: &[f64]) -> f64 {
    if nums.len() < 2 {
        return 0.0;
    }
    let n = nums.len() as f64;
    let mean = nums.iter().sum::<f64>() / n;
    let variance = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();
    if std_dev == 0.0 {
        return 0.0;
    }
    (nums.iter().map(|x| ((x - mean) / std_dev).powi(4)).sum::<f64>() / n) - 3.0
}

fn simple_stem(word: &str) -> String {
    let w = word.to_lowercase();
    for suffix in &["ing", "tion", "ness", "ment", "er", "ed", "ly", "es", "s"] {
        if w.len() > suffix.len() + 2 && w.ends_with(suffix) {
            return w[..w.len() - suffix.len()].to_string();
        }
    }
    w
}

fn setup_py_path(py: Python<'_>) -> PyResult<()> {
    let sys = PyModule::import_bound(py, "sys")?;
    let sysconfig = PyModule::import_bound(py, "sysconfig")?;
    let paths = sysconfig.call_method0("get_paths")?;
    let purelib: String = paths.get_item("purelib")?.extract()?;
    sys.getattr("path")?.call_method1("append", (purelib,))?;
    Ok(())
}
