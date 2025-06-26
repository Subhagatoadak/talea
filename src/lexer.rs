// src/lexer.rs

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // === Verbs (Commands) ===
    Load, Read, Open, Fetch, Download, Connect,
    Save, Write, Export,
    Print, Show, Display, View, Inspect, Preview, Head, Tail,
    Define, Let, Create, Set, Assign, Clear, Reset,
    Tokenize, Split, Segment, Join, Merge, Concatenate,
    Replace, Substitute, Clean, Normalize, Stem, Lemmatize,
    Lowercase, Uppercase,
    Count, Tally, Measure, Calculate, Get, Rank,
    Find, Search, Locate, Extract, Match, Filter, Keep, Remove, Exclude, Slice,
    Tag, Annotate, Concordance, Collocate, Frequency, Cluster, Correlate, Compare, Summarize,
    Sort, Order, Group,
    Add, Subtract, Multiply, Divide,
    Help, Docs, History, Run, Execute, Exit, Quit,
    Use, Python, R, Java, Scala, Ruby,Julia,

    // === Nouns (Units, Targets, Concepts) ===
    Words, Sentences, Lines, Paragraphs, Characters, Tokens, Types, Uniques,
    Length, Diversity, Readability,
    Stopwords, Punctuation, Numbers, Whitespace,
    Pattern, Regex,
    Entities, POS, NER,
    Bigrams, Trigrams, Ngrams,
    URL, JSON, CSV, XML,
    First, Last, Sample,
    Distribution, KWIC,

    // === Keywords & Prepositions ===
    As, To, From, In, Into, On, By, With,
    Containing, StartingWith, EndingWith,
    Ascending, Descending,
    Top, Bottom,

    // === Primary Types ===
    Identifier(String),
    String(String),
    Number(i64),

    // === Other ===
    Eof,
    Illegal(String),
}

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self { Lexer { input, position: 0 } }

    pub fn all_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = matches!(token, Token::Eof);
            tokens.push(token);
            if is_eof { break; }
        }
        tokens
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        if self.position >= self.input.len() { return Token::Eof; }
        let ch = self.current_char().unwrap();
        match ch {
            '"' => self.read_string(),
            _ if ch.is_alphabetic() => self.read_identifier(),
            _ if ch.is_digit(10) => self.read_number(),
            _ => { self.advance(); Token::Illegal(ch.to_string()) }
        }
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.position;
        while let Some(ch) = self.current_char() {
            if !ch.is_alphanumeric() && ch != '_' { break; }
            self.advance();
        }
        let text = &self.input[start..self.position];
        match text.to_lowercase().as_str() {
            "load"|"read"|"open" => Token::Load, "fetch"|"download" => Token::Fetch, "connect" => Token::Connect,
            "save"|"write"|"export" => Token::Save, "print"|"show"|"display"|"view" => Token::Print,
            "inspect"|"preview" => Token::Inspect, "head" => Token::Head, "tail" => Token::Tail,
            "define"|"let"|"create"|"set"|"assign" => Token::Define, "clear"|"reset" => Token::Reset,
            "tokenize"|"split"|"segment" => Token::Tokenize, "join"|"merge"|"concatenate" => Token::Join,
            "replace"|"substitute" => Token::Replace, "clean" => Token::Clean, "normalize" => Token::Normalize,
            "stem" => Token::Stem, "lemmatize" => Token::Lemmatize, "lowercase" => Token::Lowercase,
            "uppercase" => Token::Uppercase, "count"|"tally"|"measure"|"calculate" => Token::Count,
            "get" => Token::Get, "rank" => Token::Rank,
            "find"|"search"|"locate"|"extract"|"match" => Token::Find, "filter"|"keep" => Token::Filter,
            "remove"|"exclude" => Token::Remove, "slice" => Token::Slice, "tag"|"annotate" => Token::Tag,
            "concordance" => Token::Concordance, "collocate" => Token::Collocate, "frequency" => Token::Frequency,
            "cluster" => Token::Cluster, "correlate" => Token::Correlate, "compare" => Token::Compare,
            "summarize" => Token::Summarize, "sort"|"order" => Token::Sort, "group" => Token::Group,
            "add" => Token::Add, "subtract" => Token::Subtract, "multiply" => Token::Multiply, "divide" => Token::Divide,
            "help"|"docs" => Token::Help, "history" => Token::History, "run"|"execute" => Token::Run,
            "exit"|"quit" => Token::Exit, "words" => Token::Words, "sentences" => Token::Sentences,
            "lines" => Token::Lines, "paragraphs" => Token::Paragraphs, "characters" => Token::Characters,
            "tokens" => Token::Tokens, "types"|"uniques" => Token::Types, "length" => Token::Length,
            "diversity" => Token::Diversity, "readability" => Token::Readability, "stopwords" => Token::Stopwords,
            "punctuation" => Token::Punctuation, "numbers" => Token::Numbers, "whitespace" => Token::Whitespace,
            "pattern" => Token::Pattern, "regex" => Token::Regex, "entities" => Token::Entities,
            "pos" => Token::POS, "ner" => Token::NER, "bigrams" => Token::Bigrams, "trigrams" => Token::Trigrams,
            "ngrams" => Token::Ngrams, "url" => Token::URL, "json" => Token::JSON, "csv" => Token::CSV,
            "xml" => Token::XML, "first" => Token::First, "last" => Token::Last, "sample" => Token::Sample,
            "distribution" => Token::Distribution, "kwic" => Token::KWIC, "as" => Token::As, "to" => Token::To,
            "from" => Token::From, "in" => Token::In, "into" => Token::Into, "on" => Token::On, "by" => Token::By,
            "with" => Token::With, "containing" => Token::Containing, "starting_with" => Token::StartingWith,
            "ending_with" => Token::EndingWith, "ascending" => Token::Ascending, "descending" => Token::Descending,
            "top" => Token::Top, "bottom" => Token::Bottom,
            "use" => Token::Use, "python" => Token::Python, "r" => Token::R,  // New

            _ => Token::Identifier(text.to_string()),
        }
    }
    
    fn read_string(&mut self) -> Token { self.advance(); let s = self.position; while let Some(c) = self.current_char() { if c == '"' { break; } self.advance(); } let t = &self.input[s..self.position]; self.advance(); Token::String(t.to_string()) }
    fn read_number(&mut self) -> Token { let s = self.position; while let Some(c) = self.current_char() { if !c.is_digit(10) { break; } self.advance(); } let t = &self.input[s..self.position]; Token::Number(t.parse().unwrap_or(0)) }
    fn skip_whitespace(&mut self) { while let Some(c) = self.current_char() { if !c.is_whitespace() { break; } self.advance(); } }
    fn current_char(&self) -> Option<char> { self.input.chars().nth(self.position) }
    fn advance(&mut self) { if self.position < self.input.len() { self.position += 1; } }
}