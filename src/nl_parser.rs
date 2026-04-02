// src/nl_parser.rs
//
// Natural-language → Talea AST transpiler.
// No external API calls — pure corpus + regex / token-overlap matching.

use std::collections::HashMap;

use crate::ast::{Backend, Expression, FilterCondition, Statement};

// ──────────────────────────────────────────────────────────────
// Intent pattern type (fully static / embedded corpus)
// ──────────────────────────────────────────────────────────────

struct IntentPattern {
    intent: &'static str,
    patterns: &'static [&'static str],
    #[allow(dead_code)]
    example: &'static str,
}

// ──────────────────────────────────────────────────────────────
// Embedded corpus
// ──────────────────────────────────────────────────────────────

static CORPUS: &[IntentPattern] = &[
    IntentPattern {
        intent: "load_file",
        patterns: &[
            "load the file {file} and call it {var}",
            "load the file {file} and name it {var}",
            "read {file} into {var}",
            "open the document {file} and name it {var}",
            "get contents of {file} as {var}",
            "fetch the file {file} and store as {var}",
            "load {file} as {var}",
            "read {file} and store it as {var}",
            "open {file} as {var}",
            "import the file {file} and call it {var}",
            "read in {file} and save as {var}",
            "get the file {file} and name it {var}",
            "bring in {file} as {var}",
            "pull the file {file} and call it {var}",
        ],
        example: r#"load "data.txt" as my_text"#,
    },
    IntentPattern {
        intent: "load_csv",
        patterns: &[
            "load the csv file {file} and call it {dest}",
            "read the csv {file} as {dest}",
            "import the spreadsheet {file} and name it {dest}",
            "load csv {file} as {dest}",
            "open the csv {file} and call it {dest}",
            "read the spreadsheet {file} and store as {dest}",
            "load the spreadsheet {file} as {dest}",
            "import csv {file} as {dest}",
            "get the csv data from {file} and call it {dest}",
            "read data from csv {file} into {dest}",
        ],
        example: r#"load csv "data.csv" as my_data"#,
    },
    IntentPattern {
        intent: "define_var",
        patterns: &[
            "let {var} be {value}",
            "set {var} to {value}",
            "define {var} as {value}",
            "create a variable called {var} with value {value}",
            "store {value} in {var}",
            "assign {value} to {var}",
            "make {var} equal to {value}",
            "put {value} into {var}",
            "initialize {var} with {value}",
            "create {var} and set it to {value}",
            "declare {var} as {value}",
        ],
        example: "let x be 42",
    },
    IntentPattern {
        intent: "print_var",
        patterns: &[
            "print {var}",
            "show me {var}",
            "display {var}",
            "output {var}",
            "show the value of {var}",
            "print out {var}",
            "write out {var}",
            "echo {var}",
            "show {var}",
            "print the value of {var}",
            "display the contents of {var}",
        ],
        example: "print my_text",
    },
    IntentPattern {
        intent: "save",
        patterns: &[
            "save {source} to {dest}",
            "write {source} to the file {dest}",
            "export {source} to {dest}",
            "save {source} as {dest}",
            "write {source} into {dest}",
            "dump {source} to {dest}",
            "store {source} in the file {dest}",
            "persist {source} to {dest}",
        ],
        example: "save my_text to output.txt",
    },
    IntentPattern {
        intent: "tokenize",
        patterns: &[
            "break {source} into tokens and call it {dest}",
            "tokenize {source} as {dest}",
            "split {source} into tokens and store as {dest}",
            "convert {source} to tokens and name it {dest}",
            "tokenize {source} and store in {dest}",
            "break {source} into words and call it {dest}",
            "get tokens from {source} as {dest}",
            "extract tokens from {source} and call it {dest}",
            "tokenise {source} as {dest}",
        ],
        example: "tokenize my_text as tokens",
    },
    IntentPattern {
        intent: "sentiment",
        patterns: &[
            "analyze the sentiment of {source} and call it {dest}",
            "sentiment of {source} as {dest}",
            "check the mood of {source} and store in {dest}",
            "find the sentiment in {source} and name it {dest}",
            "what is the sentiment of {source} and call it {dest}",
            "detect sentiment in {source} as {dest}",
            "run sentiment analysis on {source} and store as {dest}",
            "get sentiment of {source} as {dest}",
            "analyse the sentiment of {source} as {dest}",
            "determine the mood of {source} and call it {dest}",
        ],
        example: "sentiment of my_text as my_sentiment",
    },
    IntentPattern {
        intent: "count_words",
        patterns: &[
            "count the words in {source} and call it {dest}",
            "count words in {source} as {dest}",
            "how many words are in {source} store as {dest}",
            "get the word count of {source} and call it {dest}",
            "count the number of words in {source} as {dest}",
            "find the word count of {source} and store in {dest}",
            "count tokens in {source} as {dest}",
            "get word count of {source} as {dest}",
        ],
        example: "count words in my_text as word_count",
    },
    IntentPattern {
        intent: "clean",
        patterns: &[
            "clean {source} and call it {dest}",
            "remove punctuation from {source} and store as {dest}",
            "clean up {source} as {dest}",
            "strip punctuation from {source} and call it {dest}",
            "clean the text in {source} and name it {dest}",
            "sanitize {source} as {dest}",
            "remove special characters from {source} as {dest}",
        ],
        example: "clean my_text as cleaned_text",
    },
    IntentPattern {
        intent: "lowercase",
        patterns: &[
            "convert {source} to lowercase and call it {dest}",
            "lowercase {source} as {dest}",
            "make {source} lowercase and store as {dest}",
            "turn {source} into lowercase as {dest}",
            "convert {source} to lower case as {dest}",
            "change {source} to lowercase and call it {dest}",
            "lowercase the text in {source} as {dest}",
        ],
        example: "lowercase my_text as lower_text",
    },
    IntentPattern {
        intent: "uppercase",
        patterns: &[
            "convert {source} to uppercase and call it {dest}",
            "uppercase {source} as {dest}",
            "make {source} uppercase and store as {dest}",
            "turn {source} into uppercase as {dest}",
            "change {source} to uppercase and call it {dest}",
        ],
        example: "uppercase my_text as upper_text",
    },
    IntentPattern {
        intent: "split_sentences",
        patterns: &[
            "split {source} into sentences and call it {dest}",
            "break {source} into sentences as {dest}",
            "divide {source} into sentences and store as {dest}",
            "get sentences from {source} as {dest}",
            "extract sentences from {source} and call it {dest}",
            "segment {source} into sentences as {dest}",
        ],
        example: "split my_text into sentences as sentences",
    },
    IntentPattern {
        intent: "use_python",
        patterns: &[
            "use python",
            "activate python",
            "enable python backend",
            "i need python",
            "turn on python",
            "switch to python",
            "enable python",
            "start python",
        ],
        example: "use python",
    },
    IntentPattern {
        intent: "use_r",
        patterns: &[
            "use r",
            "activate r",
            "enable r backend",
            "i need r",
            "turn on r",
            "switch to r",
            "enable r",
        ],
        example: "use r",
    },
    IntentPattern {
        intent: "for_each",
        patterns: &[
            "for each {item} in {list} do",
            "go through each {item} in {list} and",
            "loop over {item} in {list}",
            "iterate over each {item} in {list}",
            "for every {item} in {list}",
            "process each {item} in {list}",
        ],
        example: "for each word in words do",
    },
    IntentPattern {
        intent: "if_condition",
        patterns: &[
            "if {cond} then",
            "when {cond}",
            "check if {cond} and if so",
            "if {cond}",
            "given that {cond}",
        ],
        example: "if x equals 5 then",
    },
    IntentPattern {
        intent: "openai",
        patterns: &[
            "ask openai {prompt} and call it {dest}",
            "use openai to {prompt} and store as {dest}",
            "generate {prompt} with openai and call it {dest}",
            "query openai with {prompt} and store in {dest}",
            "ask gpt {prompt} and call it {dest}",
            "use gpt to {prompt} and store as {dest}",
            "get openai to {prompt} and name it {dest}",
            "prompt openai with {prompt} as {dest}",
        ],
        example: "ask openai \"Summarize this\" and call it summary",
    },
    IntentPattern {
        intent: "regress",
        patterns: &[
            "regress {target} using {features} in {source} and call it {dest}",
            "run regression of {target} on {features} in {source} as {dest}",
            "predict {target} from {features} using data in {source} as {dest}",
            "fit a regression model for {target} with {features} in {source} as {dest}",
            "build a regression model predicting {target} from {features} in {source} as {dest}",
        ],
        example: "regress price using area in data as model",
    },
    IntentPattern {
        intent: "mean",
        patterns: &[
            "find the mean of {col} in {source} and call it {dest}",
            "calculate the average of {col} in {source} as {dest}",
            "compute the mean of {col} in {source} and store as {dest}",
            "get the mean of {col} in {source} as {dest}",
            "what is the mean of {col} in {source} store as {dest}",
            "find average of {col} in {source} as {dest}",
        ],
        example: "find the mean of age in data as mean_age",
    },
    IntentPattern {
        intent: "median",
        patterns: &[
            "find the median of {col} in {source} and call it {dest}",
            "calculate the median of {col} in {source} as {dest}",
            "compute the median of {col} in {source} and store as {dest}",
            "get the median of {col} in {source} as {dest}",
        ],
        example: "find the median of age in data as median_age",
    },
    IntentPattern {
        intent: "ttest",
        patterns: &[
            "run a t-test on {col} between {g1} and {g2} and call it {dest}",
            "t-test {col} between {g1} and {g2} as {dest}",
            "compare {g1} and {g2} using a t-test on {col} as {dest}",
            "do a t-test comparing {g1} and {g2} on {col} as {dest}",
            "perform a t-test on {col} between {g1} and {g2} store as {dest}",
        ],
        example: "run a t-test on scores between group1 and group2 and call it result",
    },
    IntentPattern {
        intent: "anova",
        patterns: &[
            "run an anova on {col} between {g1} and {g2} and call it {dest}",
            "anova on {col} between {g1} and {g2} as {dest}",
            "perform anova on {col} between {g1} and {g2} store as {dest}",
        ],
        example: "run an anova on scores between group1 and group2 and call it result",
    },
    IntentPattern {
        intent: "concordance",
        patterns: &[
            "find concordance of {word} in {source} as {dest}",
            "get all sentences with {word} in {source} and call it {dest}",
            "concordance of {word} in {source} as {dest}",
            "show contexts for {word} in {source} as {dest}",
        ],
        example: "find concordance of love in my_text as love_contexts",
    },
    IntentPattern {
        intent: "wordcloud",
        patterns: &[
            "create a word cloud from {source} and call it {dest}",
            "word cloud of {source} as {dest}",
            "generate word cloud from {source} and store as {dest}",
            "build word cloud from {source} as {dest}",
            "get word frequencies in {source} as {dest}",
        ],
        example: "create a word cloud from my_text as cloud",
    },
    IntentPattern {
        intent: "stem",
        patterns: &[
            "stem {source} and call it {dest}",
            "stem the words in {source} as {dest}",
            "apply stemming to {source} and store as {dest}",
            "get stems of words in {source} as {dest}",
        ],
        example: "stem my_tokens as stemmed",
    },
    IntentPattern {
        intent: "lemmatize",
        patterns: &[
            "lemmatize {source} and call it {dest}",
            "get lemmas for {source} as {dest}",
            "apply lemmatization to {source} and store as {dest}",
            "lemmatise {source} as {dest}",
        ],
        example: "lemmatize my_tokens as lemmas",
    },
    IntentPattern {
        intent: "semantic_search",
        patterns: &[
            "search {source} for {query} and call it {dest}",
            "find sentences matching {query} in {source} as {dest}",
            "semantic search for {query} in {source} as {dest}",
            "look for {query} in {source} and store as {dest}",
        ],
        example: "search my_sentences for love and call it results",
    },
    IntentPattern {
        intent: "translate",
        patterns: &[
            "translate {source} to {language} and call it {dest}",
            "convert {source} to {language} and store as {dest}",
            "translate the text in {source} into {language} as {dest}",
        ],
        example: "translate my_text to French and call it translated",
    },
    IntentPattern {
        intent: "summarize",
        patterns: &[
            "summarize {source} and call it {dest}",
            "get a summary of {source} as {dest}",
            "give me a summary of {source} and store as {dest}",
            "create a summary of {source} as {dest}",
            "condense {source} and call it {dest}",
        ],
        example: "summarize my_text as summary",
    },
    IntentPattern {
        intent: "insight",
        patterns: &[
            "get insights from {source} and call it {dest}",
            "analyze {source} for insights as {dest}",
            "find insights in {source} and store as {dest}",
            "provide insights on {source} as {dest}",
        ],
        example: "get insights from my_data as insights",
    },
    IntentPattern {
        intent: "set_env",
        patterns: &[
            "set the environment variable {name} to {value}",
            "set env {name} to {value}",
            "define environment variable {name} as {value}",
            "set {name} environment variable to {value}",
        ],
        example: "set the environment variable OPENAI_API_KEY to my_key",
    },
];

// ──────────────────────────────────────────────────────────────
// Noise words to strip
// ──────────────────────────────────────────────────────────────

static NOISE_PHRASES: &[&str] = &[
    "please ",
    "i want to ",
    "i need to ",
    "could you ",
    "can you ",
    "would you ",
    "then ",
    "next ",
    "after that ",
    "first ",
    "finally ",
    "also ",
    "i would like to ",
    "i'd like to ",
    "kindly ",
];

// ──────────────────────────────────────────────────────────────
// Public API
// ──────────────────────────────────────────────────────────────

pub fn parse_english(text: &str) -> Result<Vec<Statement>, String> {
    let sentences = segment_sentences(text);
    let mut statements = Vec::new();

    for raw in sentences {
        let cleaned = strip_noise(&raw);
        if cleaned.trim().is_empty() {
            continue;
        }
        match match_intent(&cleaned) {
            Some((pattern, slots)) => {
                match build_statement(pattern.intent, slots) {
                    Ok(stmt) => statements.push(stmt),
                    Err(e) => eprintln!("[nl_parser] Warning: could not build statement: {}", e),
                }
            }
            None => {
                eprintln!(
                    "[nl_parser] No intent matched for: {:?}",
                    cleaned.trim()
                );
            }
        }
    }

    Ok(statements)
}

// ──────────────────────────────────────────────────────────────
// Step 1 – Sentence segmenter
// ──────────────────────────────────────────────────────────────

fn segment_sentences(text: &str) -> Vec<String> {
    // Split on sentence-ending punctuation, preserve newlines as separators too.
    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        match ch {
            '.' | '!' | '?' | '\n' => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        sentences.push(trimmed);
    }
    sentences
}

// ──────────────────────────────────────────────────────────────
// Step 2 – Noise stripper
// ──────────────────────────────────────────────────────────────

fn strip_noise(sentence: &str) -> String {
    let mut s = sentence.trim().to_lowercase();
    for phrase in NOISE_PHRASES {
        while s.starts_with(phrase) {
            s = s[phrase.len()..].trim_start().to_string();
        }
    }
    // Remove trailing noise
    let trailing = ["please", "ok", "now"];
    for t in &trailing {
        if let Some(pos) = s.rfind(t) {
            if pos > 0 {
                s = s[..pos].trim_end().to_string();
            }
        }
    }
    s
}

// ──────────────────────────────────────────────────────────────
// Step 3 – Intent matcher
// ──────────────────────────────────────────────────────────────

fn match_intent(sentence: &str) -> Option<(&'static IntentPattern, HashMap<String, String>)> {
    let mut best_score = 0.6f32; // threshold
    let mut best: Option<(&'static IntentPattern, HashMap<String, String>)> = None;

    for pattern in CORPUS {
        for &pat in pattern.patterns {
            let score = score_similarity(sentence, pat);
            if score > best_score {
                // Try to extract slots
                if let Some(slots) = extract_slots(sentence, pat) {
                    best_score = score;
                    best = Some((pattern, slots));
                }
            }
        }
    }
    best
}

// ──────────────────────────────────────────────────────────────
// Fuzzy scoring
// ──────────────────────────────────────────────────────────────

fn score_similarity(sentence: &str, pattern: &str) -> f32 {
    let pattern_words: Vec<&str> = pattern
        .split_whitespace()
        .filter(|w| !w.starts_with('{'))
        .collect();
    if pattern_words.is_empty() {
        return 0.0;
    }
    let sentence_lower = sentence.to_lowercase();
    let matches = pattern_words
        .iter()
        .filter(|w| sentence_lower.contains(&w.to_lowercase()))
        .count();
    matches as f32 / pattern_words.len() as f32
}

// ──────────────────────────────────────────────────────────────
// Step 4 – Slot extractor
// ──────────────────────────────────────────────────────────────

/// Given a sentence and a pattern like "load {file} as {var}", extract slot values.
/// Returns None if the sentence doesn't sufficiently match the pattern structure.
fn extract_slots(sentence: &str, pattern: &str) -> Option<HashMap<String, String>> {
    let mut slots: HashMap<String, String> = HashMap::new();
    let sent_lower = sentence.to_lowercase();

    // Split pattern into fixed parts and slot names
    let parts = split_pattern(pattern);

    // Build a list of (literal_text, next_slot_name) pairs
    // We walk through the sentence matching literal anchors and extracting slot values.
    let mut pos = 0usize;
    let mut i = 0usize;

    while i < parts.len() {
        match &parts[i] {
            PatternPart::Literal(lit) => {
                // Find this literal in the remaining sentence
                let lit_lower = lit.to_lowercase();
                let remaining = &sent_lower[pos..];
                match remaining.find(lit_lower.as_str()) {
                    Some(found) => {
                        pos += found + lit.len();
                        i += 1;
                    }
                    None => {
                        // Literal not found; skip (partial match still ok for scoring)
                        i += 1;
                    }
                }
            }
            PatternPart::Slot(name) => {
                // The slot spans until the next literal (or end of string)
                let end = if i + 1 < parts.len() {
                    if let PatternPart::Literal(next_lit) = &parts[i + 1] {
                        let nl = next_lit.to_lowercase();
                        let remaining = &sent_lower[pos..];
                        remaining.find(nl.as_str()).map(|f| pos + f)
                    } else {
                        None
                    }
                } else {
                    None
                };

                let raw = &sentence[pos..end.unwrap_or(sentence.len())];
                let val = raw.trim().to_string();
                if !val.is_empty() {
                    slots.insert(name.clone(), val);
                    pos = end.unwrap_or(sentence.len());
                }
                i += 1;
            }
        }
    }

    // For single-word intents (use python etc.), no slots needed
    Some(slots)
}

#[derive(Debug)]
enum PatternPart {
    Literal(String),
    Slot(String),
}

fn split_pattern(pattern: &str) -> Vec<PatternPart> {
    let mut parts = Vec::new();
    let mut remaining = pattern;

    while !remaining.is_empty() {
        if let Some(start) = remaining.find('{') {
            if start > 0 {
                parts.push(PatternPart::Literal(remaining[..start].to_string()));
            }
            if let Some(end) = remaining[start..].find('}') {
                let slot_name = remaining[start + 1..start + end].to_string();
                parts.push(PatternPart::Slot(slot_name));
                remaining = &remaining[start + end + 1..];
            } else {
                parts.push(PatternPart::Literal(remaining.to_string()));
                break;
            }
        } else {
            parts.push(PatternPart::Literal(remaining.to_string()));
            break;
        }
    }
    parts
}

// ──────────────────────────────────────────────────────────────
// Step 5 – Statement builder
// ──────────────────────────────────────────────────────────────

fn build_statement(intent: &str, slots: HashMap<String, String>) -> Result<Statement, String> {
    match intent {
        "load_file" => {
            let file = get_slot(&slots, "file")?;
            let var = get_slot(&slots, "var")?;
            Ok(Statement::Load {
                source: to_expr(&file),
                alias: Expression::Identifier(sanitize_ident(&var)),
            })
        }
        "load_csv" => {
            let file = get_slot(&slots, "file")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::LoadCsv {
                path: to_expr(&file),
                alias: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "define_var" => {
            let var = get_slot(&slots, "var")?;
            let value = get_slot(&slots, "value")?;
            Ok(Statement::Define {
                name: Expression::Identifier(sanitize_ident(&var)),
                value: parse_value_expr(&value),
            })
        }
        "print_var" => {
            let var = get_slot(&slots, "var")?;
            Ok(Statement::Print(Expression::Identifier(sanitize_ident(&var))))
        }
        "save" => {
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Save {
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: to_expr(&dest),
            })
        }
        "tokenize" => {
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Tokenize {
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "sentiment" => {
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Sentiment {
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "count_words" => {
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Count {
                unit: Expression::StringLiteral("words".to_string()),
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "clean" => {
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Clean {
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "lowercase" => {
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Lowercase {
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "uppercase" => {
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Uppercase {
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "split_sentences" => {
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Split {
                unit: Expression::StringLiteral("sentences".to_string()),
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "use_python" => Ok(Statement::Use(Backend::Python)),
        "use_r" => Ok(Statement::Use(Backend::R)),
        "for_each" => {
            let item = get_slot(&slots, "item")?;
            let list = get_slot(&slots, "list")?;
            Ok(Statement::ForEach {
                item: sanitize_ident(&item),
                list: Expression::Identifier(sanitize_ident(&list)),
                body: vec![Statement::Print(Expression::Identifier(
                    sanitize_ident(&item),
                ))],
            })
        }
        "if_condition" => {
            let cond = get_slot(&slots, "cond").unwrap_or_else(|_| "true".to_string());
            Ok(Statement::If {
                condition: parse_value_expr(&cond),
                then_body: vec![],
                else_body: vec![],
            })
        }
        "openai" => {
            let prompt = get_slot(&slots, "prompt")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::OpenAI {
                prompt: to_expr(&prompt),
                model: None,
                destination: Some(Expression::Identifier(sanitize_ident(&dest))),
                file_path: None,
            })
        }
        "regress" => {
            let target = get_slot(&slots, "target")?;
            let features = get_slot(&slots, "features")?;
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            let feature_exprs: Vec<Expression> = features
                .split(',')
                .map(|f| Expression::Identifier(sanitize_ident(f.trim())))
                .collect();
            Ok(Statement::Regress {
                target: Expression::Identifier(sanitize_ident(&target)),
                features: feature_exprs,
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
                with_openai: false,
            })
        }
        "mean" => {
            let col = get_slot(&slots, "col")?;
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Mean {
                column: Expression::StringLiteral(col),
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "median" => {
            let col = get_slot(&slots, "col")?;
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Median {
                column: Expression::StringLiteral(col),
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "ttest" => {
            let col = get_slot(&slots, "col")?;
            let g1 = get_slot(&slots, "g1")?;
            let g2 = get_slot(&slots, "g2")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::TTest {
                sample: None,
                column: Expression::StringLiteral(col),
                group1: Expression::Identifier(sanitize_ident(&g1)),
                group2: Expression::Identifier(sanitize_ident(&g2)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "anova" => {
            let col = get_slot(&slots, "col")?;
            let g1 = get_slot(&slots, "g1")?;
            let g2 = get_slot(&slots, "g2")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Anova {
                column: Expression::StringLiteral(col),
                group1: Expression::Identifier(sanitize_ident(&g1)),
                group2: Expression::Identifier(sanitize_ident(&g2)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "concordance" => {
            let word = get_slot(&slots, "word")?;
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Concordance {
                word: to_expr(&word),
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "wordcloud" => {
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::WordCloud {
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "stem" => {
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Stem {
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "lemmatize" => {
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Lemmatize {
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "semantic_search" => {
            let query = get_slot(&slots, "query")?;
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::SemanticSearch {
                query: to_expr(&query),
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "translate" => {
            let source = get_slot(&slots, "source")?;
            let language = get_slot(&slots, "language")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Translate {
                source: Expression::Identifier(sanitize_ident(&source)),
                language: Expression::StringLiteral(language),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "summarize" => {
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Summarize {
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "insight" => {
            let source = get_slot(&slots, "source")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Insight {
                source: Expression::Identifier(sanitize_ident(&source)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        "set_env" => {
            let name = get_slot(&slots, "name")?;
            let value = get_slot(&slots, "value")?;
            Ok(Statement::SetEnv {
                name: sanitize_ident(&name),
                value: to_expr(&value),
            })
        }
        "filter_containing" => {
            let source = get_slot(&slots, "source")?;
            let word = get_slot(&slots, "word")?;
            let dest = get_slot(&slots, "dest")?;
            Ok(Statement::Filter {
                source: Expression::Identifier(sanitize_ident(&source)),
                condition: FilterCondition::Containing(to_expr(&word)),
                destination: Expression::Identifier(sanitize_ident(&dest)),
            })
        }
        other => Err(format!("Unknown intent '{}'", other)),
    }
}

// ──────────────────────────────────────────────────────────────
// Helper utilities
// ──────────────────────────────────────────────────────────────

fn get_slot(slots: &HashMap<String, String>, key: &str) -> Result<String, String> {
    slots
        .get(key)
        .cloned()
        .ok_or_else(|| format!("Missing slot '{}' in matched pattern", key))
}

/// Convert a raw slot value to an Expression.
/// - "quoted string" → StringLiteral (quotes stripped)
/// - digits → Number
/// - otherwise → Identifier
fn to_expr(s: &str) -> Expression {
    let trimmed = s.trim();
    // Quoted string
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        return Expression::StringLiteral(trimmed[1..trimmed.len() - 1].to_string());
    }
    // Number
    if let Ok(n) = trimmed.parse::<i64>() {
        return Expression::Number(n);
    }
    Expression::Identifier(sanitize_ident(trimmed))
}

fn parse_value_expr(s: &str) -> Expression {
    to_expr(s)
}

/// Turn a raw word into a valid identifier (lowercase, replace spaces/hyphens with _).
fn sanitize_ident(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}
