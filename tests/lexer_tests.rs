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

#[test]
fn test_wordcloud_tokens() {
    let tokens = lex("wordcloud article as cloud");
    assert_eq!(
        tokens,
        vec![
            Token::WordCloud,
            Token::Identifier("article".to_string()),
            Token::As,
            Token::Identifier("cloud".to_string()),
            Token::Eof,
        ]
    );
}

#[test]
fn test_association_tokens() {
    let tokens = lex("association tokens as assoc");
    assert_eq!(
        tokens,
        vec![
            Token::Association,
            Token::Tokens,
            Token::As,
            Token::Identifier("assoc".to_string()),
            Token::Eof,
        ]
    );
}

#[test]
fn test_similarity_tokens() {
    let tokens = lex("similarity a to b as score");
    assert_eq!(
        tokens,
        vec![
            Token::Similarity,
            Token::Identifier("a".to_string()),
            Token::To,
            Token::Identifier("b".to_string()),
            Token::As,
            Token::Identifier("score".to_string()),
            Token::Eof,
        ]
    );
}

#[test]
fn test_semanticsearch_tokens() {
    let tokens = lex("semanticsearch \"hi\" in documents as results");
    assert_eq!(
        tokens,
        vec![
            Token::SemanticSearch,
            Token::String("hi".to_string()),
            Token::In,
            Token::Identifier("documents".to_string()),
            Token::As,
            Token::Identifier("results".to_string()),
            Token::Eof,
        ]
    );
}

#[test]
fn test_search_tokens() {
    let tokens = lex("search \"rust\" in article as hits");
    assert_eq!(
        tokens,
        vec![
            Token::Search,
            Token::String("rust".to_string()),
            Token::In,
            Token::Identifier("article".to_string()),
            Token::As,
            Token::Identifier("hits".to_string()),
            Token::Eof,
        ]
    );
}

#[test]
fn test_shapiro_tokens() {
    let tokens = lex("shapiro column height in data as result");
    assert_eq!(
        tokens,
        vec![
            Token::Shapiro,
            Token::Identifier("column".to_string()),
            Token::Identifier("height".to_string()),
            Token::In,
            Token::Identifier("data".to_string()),
            Token::As,
            Token::Identifier("result".to_string()),
            Token::Eof,
        ]
    );
}

#[test]
fn test_kolmogorov_tokens() {
    let tokens = lex("kolmogorov column height in data as result");
    assert_eq!(
        tokens,
        vec![
            Token::Kolmogorov,
            Token::Identifier("column".to_string()),
            Token::Identifier("height".to_string()),
            Token::In,
            Token::Identifier("data".to_string()),
            Token::As,
            Token::Identifier("result".to_string()),
            Token::Eof,
        ]
    );
}
