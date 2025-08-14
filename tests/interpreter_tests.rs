use talea::lexer::Lexer;
use talea::parser::Parser;
use talea::runtime::TaleaValue;
use talea::runtime::interpreter::Interpreter;

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
