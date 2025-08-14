use serde_json::Value;
use talea::lexer::Lexer;
use talea::parser::Parser;
use talea::runtime::interpreter::Interpreter;
use talea::runtime::{TaleaValue, load_talea_object, save_talea_object};

fn run_script(input: &str) -> Result<Interpreter, String> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.all_tokens();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let mut interpreter = Interpreter::new();
    interpreter.execute(statements)?;
    Ok(interpreter)
}

#[test]
fn test_interpreter_load_tokenize_split() {
    let script = r#"
        load "article.txt" as article
        tokenize article as tokens
        split sentences in article as sentences
    "#;
    let interp = run_script(script).unwrap();

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
    let interp = run_script(script).unwrap();

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
    let interp = run_script(script).unwrap();
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
    let interp = run_script(script).unwrap();
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
    let interp = run_script(script).unwrap();
    match interp.get_variable("hits") {
        Some(TaleaValue::List(list)) => {
            assert_eq!(list.len(), 1);
            assert_eq!(list[0], TaleaValue::String("hello world".to_string()));
        }
        _ => panic!("hits not found"),
    }
}

#[test]
fn test_save_and_load_talea_object() {
    let value = TaleaValue::List(vec![
        TaleaValue::Number(1),
        TaleaValue::String("two".to_string()),
    ]);
    let path = "tests/data/temp_talea_object.json";
    save_talea_object(&value, path).unwrap();
    let loaded = load_talea_object(path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(value, loaded);
}

#[test]
fn test_python_dataframe_conversion() {
    let script = r#"
        use python
        install python "pandas"
        import python "pandas" as pd
        call pd read_csv "examples/sample_data.csv" as data
    "#;
    let interp = match run_script(script) {
        Ok(i) => i,
        Err(e)
            if e.contains("pip install pandas")
                || e.contains("Failed to install Python module") =>
        {
            println!("skipping test: {e}");
            return;
        }
        Err(e) => panic!("unexpected error: {e}"),
    };
    match interp.get_variable("data") {
        Some(TaleaValue::PyObject(_)) => {}
        other => panic!("expected PyObject, got {:?}", other),
    }
}

#[test]
fn test_callmethod_describe() {
    let script = r#"
        use python
        install python "pandas"
        import python "pandas" as pd
        call pd read_csv "examples/sample_data.csv" as data
        callmethod data describe as summary
    "#;
    let interp = match run_script(script) {
        Ok(i) => i,
        Err(e)
            if e.contains("pip install pandas")
                || e.contains("Failed to install Python module") =>
        {
            println!("skipping test: {e}");
            return;
        }
        Err(e) => panic!("unexpected error: {e}"),
    };
    assert!(interp.get_variable("summary").is_some());
}

#[test]
fn test_callmethod_dunder() {
    let script = r#"
        use python
        install python "pandas"
        import python "pandas" as pd
        call pd read_csv "examples/sample_data.csv" as data
        callmethod data __getitem__ "y" as column
    "#;
    let interp = match run_script(script) {
        Ok(i) => i,
        Err(e)
            if e.contains("pip install pandas")
                || e.contains("Failed to install Python module") =>
        {
            println!("skipping test: {e}");
            return;
        }
        Err(e) => panic!("unexpected error: {e}"),
    };
    assert!(interp.get_variable("column").is_some());
}

#[test]
fn test_expression_statement() {
    let script = r#"
        define x as 5
        x
    "#;
    let interp = run_script(script).unwrap();
    match interp.get_variable("x") {
        Some(TaleaValue::Number(n)) => assert_eq!(n, 5),
        other => panic!("expected 5, got {:?}", other),
    }
}

#[test]
fn test_callmethod_non_pyobject_error() {
    let script = r#"
        define x as 1
        callmethod x foo as y
    "#;
    let err = run_script(script).err().expect("expected error");
    assert!(err.contains("not a Python object"));
}

#[test]
fn test_regress_outputs_metrics() {
    let script = r#"
        use python
        install python "pandas"
        install python "statsmodels"
        install python "matplotlib"
        load "examples/sample_data.csv" as data
        regress y using x in data as model
    "#;
    let interp = match run_script(script) {
        Ok(i) => i,
        Err(e)
            if e.contains("pip install pandas")
                || e.contains("pip install statsmodels")
                || e.contains("pip install matplotlib")
                || e.contains("Failed to install Python module")
                || e.contains("ModuleNotFoundError")
                || e.contains("No module named") =>
        {
            println!("skipping test: {e}");
            return;
        }
        Err(e) => panic!("unexpected error: {e}"),
    };
    let result = match interp.get_variable("model") {
        Some(TaleaValue::String(s)) => s,
        other => panic!("expected string result, got {:?}", other),
    };
    let parsed: Value = serde_json::from_str(&result).expect("valid json");
    assert!(parsed.get("coefficients").is_some());
    assert!(parsed.get("predictions").is_some());
    assert!(parsed.get("metrics").is_some());
    let metrics = parsed.get("metrics").unwrap();
    assert!(metrics.get("r_squared").is_some());
    assert!(metrics.get("mape").is_some());
    assert!(metrics.get("rmse").is_some());
    assert!(parsed.get("chart").is_some());
}
