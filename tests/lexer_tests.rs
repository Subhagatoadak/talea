use talea::lexer::{Lexer, Token};

fn lex(input: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(input);
    lexer.all_tokens()
}

#[test]
fn test_load_tokens() {
    let tokens = lex("load \"article.txt\" as article");
    assert_eq!(
        tokens,
        vec![
            Token::Load,
            Token::String("article.txt".to_string()),
            Token::As,
            Token::Identifier("article".to_string()),
            Token::Eof,
        ]
    );
}

#[test]
fn test_tokenize_tokens() {
    let tokens = lex("tokenize article as tokens");
    assert_eq!(
        tokens,
        vec![
            Token::Tokenize,
            Token::Identifier("article".to_string()),
            Token::As,
            Token::Tokens,
            Token::Eof,
        ]
    );
}

#[test]
fn test_split_tokens() {
    let tokens = lex("split sentences in article as sentences");
    assert_eq!(
        tokens,
        vec![
            Token::Split,
            Token::Sentences,
            Token::In,
            Token::Identifier("article".to_string()),
            Token::As,
            Token::Sentences,
            Token::Eof,
        ]
    );
}
