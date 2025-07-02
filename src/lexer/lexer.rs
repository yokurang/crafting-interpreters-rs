use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::vec::Vec;
use crate::utils::{error};

pub static KEYWORDS: Lazy<HashMap<&'static str, TokenType>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("and", TokenType::And);
    m.insert("class", TokenType::Class);
    m.insert("else", TokenType::Else);
    m.insert("false", TokenType::False);
    m.insert("for", TokenType::For);
    m.insert("fun", TokenType::Fun);
    m.insert("if", TokenType::If);
    m.insert("nil", TokenType::Nil);
    m.insert("or", TokenType::Or);
    m.insert("print", TokenType::Print);
    m.insert("return", TokenType::Return);
    m.insert("super", TokenType::Super);
    m.insert("this", TokenType::This);
    m.insert("true", TokenType::True);
    m.insert("var", TokenType::Var);
    m.insert("while", TokenType::While);
    m
});

/*
The scanner's job is to scan source code as a sequence of characters and group sequences of
characters together into lexemes. Such lexemes are then evaluated into tokens for later analysis.

Tokens are individual atoms in the molecule that is a programming language. Each lexeme is a sequence
of characters, maps to a particular token. We need a token for every atomic structure of the programming language
as per the language specification.
*/

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    // single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    SemiColon,
    Slash,
    Star,

    // one or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    /* Literals:
    Literals are tokens that represent the value of their textual representation.
    This is helpful for the runtime object, so it knows that the token can be evaluated
    to the value of its textual representation.
    */
    Identifier,
    String,
    Number,

    // keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/*
In error handling, this interpreter only keeps track of the line where the error occurred.
In a more sophisticated programming language, it would also keep track of the column and length of the token.
Most implementations do this by keeping track of two variables: the offset position from the beginning
of the source file to the line at which an error occurred, and the length of the lexeme.
The row and column positions can be inferred from these two variables.
*/

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: Literal, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} {} {:?}", self.token_type, self.lexeme, self.literal)
    }
}

#[derive(Debug, Clone, PartialEq)] // PartialEq is fine for deriving
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

// Implementing Eq for Literal enum
impl Eq for Literal {}

// Implementing Hash for Literal enum
impl std::hash::Hash for Literal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Literal::String(s) => s.hash(state),
            Literal::Number(n) => {
                // Convert f64 to bits for hashing
                n.to_bits().hash(state)
            },
            Literal::Bool(b) => b.hash(state),
            Literal::Nil => 0.hash(state),
        }
    }
}

/* Scanner:
The scanner works by consuming the character from the source code, identifying what lexeme
the character belongs to, and continue consuming any other character that belongs to that lexeme.
Once the end of the lexeme is reached, the scanner emits a token. The scanner then loops to the start of the process
of identifying lexemes again and consumes characters from the source code until it reaches the end of the file, at which an
EOF condition is signaled.
*/

/*
The rules that determine how a particular language groups a sequence of characters into lexemes
are called its lexical grammar.
*/

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    // these fields are used by the scanner to keep track of its position in the input
    start: usize,   // points to the first position in the lexeme
    current: usize, // points to the current position of the lexeme
    line: usize, // keeps track which source line `current` is on so we can print out the location of the tokens
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::<Token>::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token::new(
            TokenType::Eof,
            "".to_string(),
            Literal::Nil,
            self.line,
        ));
        &self.tokens
    }

    // to consume input
    fn advance(&mut self) -> char {
        let ch = self.source[self.current..].chars().next().unwrap();
        self.current += ch.len_utf8();
        ch
    }

    // in scanning a token, if the token is a single character long, all we need to do is consume
    // the character and map it its respective token

    fn scan_token(&mut self) {
        let ch = self.advance();
        match ch {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::SemiColon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let token = if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token);
            }
            '=' => {
                let token = if self.match_char('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token);
            }
            '<' => {
                let token = if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token);
            }
            '>' => {
                let token = if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token);
            }
            '/' => {
                // the rules of the lexical grammar determine how much lookahead we need
                if self.match_char('/') {
                    // a comment goes until the line's end
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => {
                // we still want to get here to increment `self.line`. That's why we use
                // `peek()` instead of `match()`.
                self.line += 1;
            }
            // maximal munch is when a sequence of characters can match to two or more possible tokens.
            // the sequence of characters will match to the token with the most number of character matches.
            '"' => self.string(),
            c => {
                if self.is_digit(c) {
                    self.number();
                } else if self.is_alpha(c) {
                    self.identifier();
                } else {
                    // if an unexpected character is consumed, throw an error
                    // note that the erroneous character is still consumed by `advance()`.
                    // This is important to avoid an infinite loop.
                    // Since HAD_ERROR will be set to true, we never execute the code,
                    // but we keep scanning through the source code to catch all the errors at once
                    error(self.line, "Unexpected character.");
                }
            }
        }
    }

    fn is_alpha(&self, c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }

    fn is_alphanumeric(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }

    fn is_digit(&self, ch: char) -> bool {
        ch >= '0' && ch <= '9'
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source[self.current..].chars().next().unwrap()
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        // chars returns an iterator, so to get the first character, we need to call `next()`
        self.source[self.current..].chars().next().unwrap()
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            false;
        }

        let next_char = self.source[self.current..].chars().next().unwrap();
        if next_char != expected {
            false;
        }
        self.current += next_char.len_utf8();
        true
    }

    fn string(&mut self) -> () {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            error(self.line, "Unterminated string.");
            return;
        }

        // the closing "
        self.advance();
        let value = &self.source[self.start + 1..self.current - 1];
        self.add_token_with_literal(TokenType::String, Literal::String(value.to_string()));
    }

    fn number(&mut self) -> () {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();
            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        let text = &self.source[self.start..self.current];
        let value: f64 = text.parse().unwrap();
        self.add_token_with_literal(TokenType::Number, Literal::Number(value));
    }

    fn identifier(&mut self) {
        while self.is_alphanumeric(self.peek()) {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let token_type = KEYWORDS.get(text).cloned().unwrap_or(TokenType::Identifier);
        self.add_token(token_type);
    }

    // to produce output
    fn add_token(&mut self, token_type: TokenType) -> () {
        self.add_token_with_literal(token_type, Literal::Nil);
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Literal) -> () {
        let text = self.source[self.start..self.current].to_string();
        let token = Token::new(token_type, text, literal, self.line);
        self.tokens.push(token);
    }
}
