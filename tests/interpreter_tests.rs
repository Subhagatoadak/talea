use talea::lexer::Lexer;
use talea::parser::Parser;
use talea::runtime::TaleaValue;
use talea::runtime::interpreter::Interpreter;
use pyo3::prelude::*;

fn run_script(input: &str) -> Interpreter {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.all_tokens();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let mut interpreter = Interpreter::new();
    interpreter.execute(statements).unwrap();
    interpreter
}

#[test]
fn test_interpreter_load_tokenize_split() {
    let script = r#"
        load "article.txt" as article
        tokenize article as tokens
        split sentences in article as sentences
    "#;
    let interp = run_script(script);

    match interp.get_variable("article") {
        Some(TaleaValue::String(s)) => assert!(s.contains("India is the best")),
        _ => panic!("article not loaded"),
    }

    match interp.get_variable("tokens") {
        Some(TaleaValue::List(list)) => assert_eq!(list.len(), 13),
        _ => panic!("tokens not generated"),
    }

    match interp.get_variable("sentences") {
        Some(TaleaValue::List(list)) => assert_eq!(list.len(), 2),
        _ => panic!("sentences not split"),
    }
}

#[test]
fn test_doc2bow() {
    let script = r#"
        load "article.txt" as article
        tokenize article as tokens
        doc2bow tokens as bow
    "#;
    let interp = run_script(script);

    match interp.get_variable("bow") {
        Some(TaleaValue::List(list)) => {
            assert_eq!(list.len(), 12);
            let mut found_the = false;
            for item in list {
                if let TaleaValue::Tuple(tuple) = item {
                    if let [TaleaValue::String(word), TaleaValue::Number(count)] = &tuple[..] {
                        if word == "the" && *count == 2 {
                            found_the = true;
                        }
                    }
                }
            }
            assert!(found_the, "expected 'the' with count 2");
        }
        _ => panic!("bow not generated"),
    }
}

#[test]
fn test_wordcloud_command() {
    let script = r#"
        wordcloud "hello world hello" as cloud
    "#;
    let interp = run_script(script);
    match interp.get_variable("cloud") {
        Some(TaleaValue::List(list)) => {
            assert_eq!(list.len(), 2);
            if let TaleaValue::Tuple(first) = &list[0] {
                assert_eq!(first[0], TaleaValue::String("hello".to_string()));
                assert_eq!(first[1], TaleaValue::Number(2));
            } else {
                panic!("expected tuple");
            }
        }
        _ => panic!("cloud not generated"),
    }
}

#[test]
fn test_similarity_command() {
    let script = r#"
        similarity "hello world" to "hello there" as score
    "#;
    let interp = run_script(script);
    match interp.get_variable("score") {
        Some(TaleaValue::Number(n)) => assert_eq!(n, 33),
        _ => panic!("score not computed"),
    }
}

#[test]
fn test_search_command() {
    let script = r#"
        search "hello" in "hello world. hi there." as hits
    "#;
    let interp = run_script(script);
    match interp.get_variable("hits") {
        Some(TaleaValue::List(list)) => {
            assert_eq!(list.len(), 1);
            assert_eq!(list[0], TaleaValue::String("hello world".to_string()));
        }
        _ => panic!("hits not found"),
    }
}

#[test]
fn test_load_csv_command() {
    if Python::with_gil(|py| PyModule::import_bound(py, "pandas").is_err()) {
        eprintln!("pandas not available, skipping test");
        return;
    }
    let script = r#"
        use python
        load csv "tests/data/sample.csv" as data
    "#;
    let interp = run_script(script);
    match interp.get_variable("data") {
        Some(TaleaValue::Table(rows)) => {
            assert_eq!(rows.len(), 2);
            assert_eq!(
                rows[0].get("name"),
                Some(&TaleaValue::String("Alice".to_string()))
            );
        }
        _ => panic!("data not loaded"),
    }
}
