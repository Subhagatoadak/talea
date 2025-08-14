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
