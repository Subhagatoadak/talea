use talea::ast::{Expression, Statement};
use talea::lexer::{Lexer, Token};
use talea::parser::Parser;

fn parse(input: &str) -> Vec<Statement> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.all_tokens();
    let mut parser = Parser::new(tokens);
    parser.parse().unwrap()
}

#[test]
fn test_parse_load() {
    let statements = parse("load \"article.txt\" as article");
    assert_eq!(
        statements[0],
        Statement::Load {
            source: Expression::StringLiteral("article.txt".to_string()),
            alias: Expression::Identifier("article".to_string()),
        }
    );
}

#[test]
fn test_parse_load_csv() {
    let statements = parse("load csv \"tests/data/sample.csv\" as data");
    assert_eq!(
        statements[0],
        Statement::LoadCsv {
            path: Expression::StringLiteral("tests/data/sample.csv".to_string()),
            alias: Expression::Identifier("data".to_string()),
        }
    );
}

#[test]
fn test_parse_tokenize() {
    let statements = parse("tokenize article as tokens");
    assert_eq!(
        statements[0],
        Statement::Tokenize {
            source: Expression::Identifier("article".to_string()),
            destination: Expression::Identifier("tokens".to_string()),
        }
    );
}

#[test]
fn test_parse_split() {
    let statements = parse("split sentences in article as sentences");
    assert_eq!(
        statements[0],
        Statement::Split {
            unit: Expression::Unit(Token::Sentences),
            source: Expression::Identifier("article".to_string()),
            destination: Expression::Identifier("sentences".to_string()),
        }
    );
}

#[test]
fn test_parse_wordcloud() {
    let statements = parse("wordcloud article as cloud");
    assert_eq!(
        statements[0],
        Statement::WordCloud {
            source: Expression::Identifier("article".to_string()),
            destination: Expression::Identifier("cloud".to_string()),
        }
    );
}

#[test]
fn test_parse_association() {
    let statements = parse("association tokens as assoc");
    assert_eq!(
        statements[0],
        Statement::WordAssociation {
            source: Expression::Identifier("tokens".to_string()),
            destination: Expression::Identifier("assoc".to_string()),
        }
    );
}

#[test]
fn test_parse_similarity() {
    let statements = parse("similarity a to b as score");
    assert_eq!(
        statements[0],
        Statement::SemanticSimilarity {
            text1: Expression::Identifier("a".to_string()),
            text2: Expression::Identifier("b".to_string()),
            destination: Expression::Identifier("score".to_string()),
        }
    );
}

#[test]
fn test_parse_semanticsearch() {
    let statements = parse("semanticsearch \"hi\" in documents as results");
    assert_eq!(
        statements[0],
        Statement::SemanticSearch {
            query: Expression::StringLiteral("hi".to_string()),
            source: Expression::Identifier("documents".to_string()),
            destination: Expression::Identifier("results".to_string()),
        }
    );
}

#[test]
fn test_parse_search() {
    let statements = parse("search \"rust\" in article as hits");
    assert_eq!(
        statements[0],
        Statement::KeywordSearch {
            query: Expression::StringLiteral("rust".to_string()),
            source: Expression::Identifier("article".to_string()),
            destination: Expression::Identifier("hits".to_string()),
        }
    );
}

#[test]
fn test_parse_callmethod() {
    let statements = parse("callmethod data describe as summary");
    assert_eq!(
        statements[0],
        Statement::CallMethod {
            object: Expression::Identifier("data".to_string()),
            method: Expression::Identifier("describe".to_string()),
            args: vec![],
            destination: Expression::Identifier("summary".to_string()),
        }
    );
}

#[test]
fn test_parse_callmethod_dunder() {
    let statements = parse("callmethod data __getitem__ 0 as first");
    assert_eq!(
        statements[0],
        Statement::CallMethod {
            object: Expression::Identifier("data".to_string()),
            method: Expression::Identifier("__getitem__".to_string()),
            args: vec![Expression::Number(0)],
            destination: Expression::Identifier("first".to_string()),
        }
    );
}

#[test]
fn test_parse_expression_statement() {
    let statements = parse("data");
    assert_eq!(
        statements[0],
        Statement::Expression(Expression::Identifier("data".to_string()))
    );
}

#[test]
fn test_parse_compare() {
    let statements = parse("compare a to b as result");
    assert_eq!(
        statements[0],
        Statement::Compare {
            left: Expression::Identifier("a".to_string()),
            right: Expression::Identifier("b".to_string()),
            destination: Expression::Identifier("result".to_string()),
        }
    );
}

#[test]
fn test_parse_average() {
    let statements = parse("average numbers as avg_result");
    assert_eq!(
        statements[0],
        Statement::Average {
            source: Expression::Identifier("numbers".to_string()),
            destination: Expression::Identifier("avg_result".to_string()),
        }
    );
}

#[test]
fn test_parse_regress() {
    let statements = parse("regress salary using age, education in data as model");
    assert_eq!(
        statements[0],
        Statement::Regress {
            target: Expression::Identifier("salary".to_string()),
            features: vec![
                Expression::Identifier("age".to_string()),
                Expression::Identifier("education".to_string()),
            ],
            source: Expression::Identifier("data".to_string()),
            destination: Expression::Identifier("model".to_string()),
            with_openai: false,
        }
    );
}

#[test]
fn test_parse_shapiro() {
    let statements = parse("shapiro column height in data as result");
    assert_eq!(
        statements[0],
        Statement::Shapiro {
            column: Expression::Identifier("height".to_string()),
            data: Expression::Identifier("data".to_string()),
            destination: Expression::Identifier("result".to_string()),
        }
    );
}

#[test]
fn test_parse_check_normality() {
    let statements = parse("check normality numbers as is_normal");
    assert_eq!(
        statements[0],
        Statement::CheckNormality {
            source: Expression::Identifier("numbers".to_string()),
            destination: Expression::Identifier("is_normal".to_string()),
        }
    );
}

#[test]
fn test_parse_kolmogorov() {
    let statements = parse("kolmogorov column height in data as result");
    assert_eq!(
        statements[0],
        Statement::Kolmogorov {
            column: Expression::Identifier("height".to_string()),
            data: Expression::Identifier("data".to_string()),
            destination: Expression::Identifier("result".to_string()),
        }
    );
}

#[test]
fn test_parse_ignores_comments() {
    let script = "define x as 1 # inline comment\n# full line\nprint x";
    let statements = parse(script);
    assert_eq!(statements.len(), 2);
}
