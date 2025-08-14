use talea::ast::{Expression, Statement};
use talea::lexer::Lexer;
use talea::lexer::Token;
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
fn test_parse_ttest() {
    let statements = parse("two sample ttest column score between group1 and group2 as t_result");
    assert_eq!(
        statements[0],
        Statement::TTest {
            sample: Some("two".to_string()),
            column: Expression::Identifier("score".to_string()),
            group1: Expression::Identifier("group1".to_string()),
            group2: Expression::Identifier("group2".to_string()),
            destination: Expression::Identifier("t_result".to_string()),
        }
    );
}
