// src/lexer.rs

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Commands
    Load,
    Print,

    // Keywords
    As,
    From,

    // Primary
    Identifier(String), // e.g., my_data
    String(String),   // e.g., "hamlet.txt"

    // Other
    Eof,       // End of File
    Illegal(String),   // An illegal/unknown token
}

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer { input, position: 0 }
    }

    pub fn all_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = matches!(token, Token::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return Token::Eof;
        }

        let ch = self.current_char().unwrap();

        match ch {
            '"' => self.read_string(),
            _ if ch.is_alphabetic() => self.read_identifier(),
            _ => {
                self.advance();
                Token::Illegal(ch.to_string())
            }
        }
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.position;
        while let Some(ch) = self.current_char() {
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            self.advance();
        }
        let text = &self.input[start..self.position];
        match text {
            "load" => Token::Load,
            "print" => Token::Print,
            "as" => Token::As,
            "from" => Token::From,
            _ => Token::Identifier(text.to_string()),
        }
    }

    fn read_string(&mut self) -> Token {
        self.advance(); // consume opening "
        let start = self.position;
        while let Some(ch) = self.current_char() {
            if ch == '"' {
                break;
            }
            self.advance();
        }
        let text = &self.input[start..self.position];
        self.advance(); // consume closing "
        Token::String(text.to_string())
    }
    
    // Helper methods
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if !ch.is_whitespace() {
                break;
            }
            self.advance();
        }
    }
    
    fn current_char(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    fn advance(&mut self) {
        if self.position < self.input.len() {
            self.position += 1;
        }
    }
}