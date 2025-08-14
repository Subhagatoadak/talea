use talea::lexer::Lexer;
use talea::parser::Parser;
use talea::runtime::TaleaValue;
use talea::runtime::interpreter::Interpreter;
use pyo3::prelude::*;

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

fn test_ttest_command() {
    let script = r#"
        use python
        define group1 as [1,2,3,4]
        define group2 as [2,3,4,5]
        two sample ttest column score between group1 and group2 as t_result
    "#;
    let interp = run_script(script);
    match interp.get_variable("t_result") {
        Some(TaleaValue::Tuple(t)) => assert_eq!(t.len(), 2),
        _ => panic!("t_result not computed"),


fn test_shapiro_command() {
    if std::process::Command::new("python")
        .args(["-c", "import scipy"])
        .status()
        .map(|s| !s.success())
        .unwrap_or(true)
    {
        eprintln!("scipy not available, skipping shapiro test");
        return;
    }
    let script = r#"
        define data as [["height", [1,2,3,4,5,6]]]
        use python
        install python "numpy"
        install python "scipy"
        shapiro column height in data as result
    "#;
    let interp = match run_script(script) {
        Ok(i) => i,
        Err(_) => {
            eprintln!("scipy not available, skipping shapiro test");
            return;
        }
    };
    match interp.get_variable("result") {
        Some(TaleaValue::Tuple(vals)) => {
            if let [TaleaValue::Float(stat), TaleaValue::Float(p)] = &vals[..] {
                assert!((*stat - 0.9818894288744925).abs() < 1e-6);
                assert!((*p - 0.9605549608523386).abs() < 1e-6);
            } else {
                panic!("expected tuple of floats");
            }
        }
        _ => panic!("result not found"),
    }
}

#[test]
fn test_kolmogorov_command() {
    if std::process::Command::new("python")
        .args(["-c", "import scipy"])
        .status()
        .map(|s| !s.success())
        .unwrap_or(true)
    {
        eprintln!("scipy not available, skipping kolmogorov test");
        return;
    }
    let script = r#"
        define data as [["height", [1,2,3,4,5,6]]]
        use python
        install python "numpy"
        install python "scipy"
        kolmogorov column height in data as result
    "#;
    let interp = match run_script(script) {
        Ok(i) => i,
        Err(_) => {
            eprintln!("scipy not available, skipping kolmogorov test");
            return;
        }
    };
    match interp.get_variable("result") {
        Some(TaleaValue::Tuple(vals)) => {
            if let [TaleaValue::Float(stat), TaleaValue::Float(p)] = &vals[..] {
                assert!((*stat - 0.8413447460685429).abs() < 1e-6);
                assert!((*p - 3.189750738921233e-05).abs() < 1e-10);
            } else {
                panic!("expected tuple of floats");
            }
        }
        _ => panic!("result not found"),

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
