use std::{iter::FromIterator, str::FromStr};

use crate::error::{Error, Result};

const STRING_INITIALIZATION_CAPACITY: usize = 8;

// TODO: '_' in long integer numbers?
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Integer(i64),
    Float(f64),
    Rational(i64, i64),
    String(String),
    Boolean(bool),
    Word(String),
    Plus,
    Minus,
    Star,
    Slash,
    EqualEqual,
    BangEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Bang,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Colon,
    Semicolon,
    LongDash,
    Comment,
    DocComment(String),
    EOF,
}

impl TokenType {
    fn parse_multiplier(word: &str) -> (&str, i64) {
        if let Some(stripped) = word.strip_prefix('-') {
            (stripped, -1)
        } else if let Some(stripped) = word.strip_prefix('+') {
            (stripped, 1)
        } else {
            (word, 1)
        }
    }

    fn parse_rational(multiplier: i64, number_word: &str) -> Option<TokenType> {
        if let Some((left, denominator)) = number_word.split_once('/') {
            if let Ok(denominator) = denominator.parse::<i64>() {
                let signum = if denominator < 0 { -1 } else { 1 };

                let (whole_number, numerator) = if let Some((whole, numer)) = left.split_once('+') {
                    (
                        whole.parse::<i64>().unwrap_or(0),
                        numer.parse::<i64>().unwrap_or(0),
                    )
                } else if let Some((whole, numer)) = left.split_once('-') {
                    (
                        whole.parse::<i64>().unwrap_or(0),
                        -numer.parse::<i64>().unwrap_or(0),
                    )
                } else {
                    (0, left.parse::<i64>().unwrap_or(0))
                };

                let (numerator, denominator) = if whole_number == 0 {
                    (signum * multiplier * numerator, denominator.abs())
                } else {
                    (
                        signum * multiplier * whole_number * denominator + numerator,
                        denominator.abs(),
                    )
                };

                return Some(TokenType::Rational(numerator, denominator));
            }
        }

        None
    }

    fn parse_with_radix(
        multiplier: i64,
        number_word: &str,
        prefix1: &str,
        prefix2: &str,
        radix: u32,
    ) -> Option<TokenType> {
        if number_word.starts_with(prefix1) || number_word.starts_with(prefix2) {
            let int_str = number_word
                .trim_start_matches(prefix1)
                .trim_start_matches(prefix2);
            if let Ok(number) = i64::from_str_radix(int_str, radix) {
                return Some(TokenType::Integer(multiplier * number));
            }
        }
        None
    }
}

// TODO: The smarter parts of this should probably go in the compiler.
// TODO: can this fail? should this be an instance of From instead?
impl FromStr for TokenType {
    type Err = Error;

    fn from_str(word: &str) -> Result<Self> {
        // Simple words
        match word {
            "+" => return Ok(TokenType::Plus),
            "-" => return Ok(TokenType::Minus),
            "*" => return Ok(TokenType::Star),
            "/" => return Ok(TokenType::Slash),
            "true" => return Ok(TokenType::Boolean(true)),
            "false" => return Ok(TokenType::Boolean(false)),
            "==" => return Ok(TokenType::EqualEqual),
            "!=" => return Ok(TokenType::BangEqual),
            "<" => return Ok(TokenType::Less),
            ">" => return Ok(TokenType::Greater),
            "<=" => return Ok(TokenType::LessEqual),
            ">=" => return Ok(TokenType::GreaterEqual),
            "!" => return Ok(TokenType::Bang),
            "{" => return Ok(TokenType::OpenBrace),
            "}" => return Ok(TokenType::CloseBrace),
            "(" => return Ok(TokenType::OpenParen),
            ")" => return Ok(TokenType::CloseParen),
            "[" => return Ok(TokenType::OpenBracket),
            "]" => return Ok(TokenType::CloseBracket),
            ":" => return Ok(TokenType::Colon),
            ";" => return Ok(TokenType::Semicolon),
            "--" => return Ok(TokenType::LongDash),
            _ => {}
        }

        // non-base-10 numbers and rationals
        let (number_word, multiplier) = TokenType::parse_multiplier(word);

        if let Some(hex) = TokenType::parse_with_radix(multiplier, number_word, "0x", "0X", 16) {
            return Ok(hex);
        } else if let Some(oct) =
            TokenType::parse_with_radix(multiplier, number_word, "0o", "0O", 8)
        {
            return Ok(oct);
        } else if let Some(bin) =
            TokenType::parse_with_radix(multiplier, number_word, "0b", "0B", 2)
        {
            return Ok(bin);
        } else if number_word.starts_with(|c| char::is_digit(c, 10)) && number_word.contains('/') {
            if let Some(rational) = TokenType::parse_rational(multiplier, number_word) {
                return Ok(rational);
            }
        }

        // Things Rust can parse.
        if let Ok(number) = number_word.parse::<i64>() {
            return Ok(TokenType::Integer(number * multiplier));
        } else if let Ok(number) = number_word.parse::<f64>() {
            return Ok(TokenType::Float(number * multiplier as f64));
        }

        // Strings
        if word.starts_with("\"") {
            return Ok(TokenType::String(word.trim_matches('"').to_string()));
        }

        Ok(TokenType::Word(word.to_string()))
    }
}

impl From<i64> for TokenType {
    fn from(value: i64) -> Self {
        TokenType::Integer(value)
    }
}

impl From<f64> for TokenType {
    fn from(value: f64) -> Self {
        TokenType::Float(value)
    }
}

impl From<String> for TokenType {
    fn from(value: String) -> Self {
        TokenType::String(value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub start: usize,
    pub line_no: usize,
    pub column: usize,
    pub length: usize,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        start: usize,
        line_no: usize,
        column: usize,
        length: usize,
    ) -> Self {
        Self {
            token_type,
            start,
            line_no,
            column,
            length,
        }
    }

    pub fn from_token_type(input: &str) -> Result<Self> {
        Ok(Token::new(input.parse()?, 0, 0, 0, 0))
    }
}

#[derive(Debug, Copy, Clone)]
struct Pos {
    index: usize,
    line_no: usize,
    column: usize,
}

impl Pos {
    fn new(index: usize, line_no: usize, column: usize) -> Self {
        Pos {
            index,
            line_no,
            column,
        }
    }

    fn from_scanner(scanner: &Scanner) -> Self {
        Pos {
            index: scanner.index - 1,
            line_no: scanner.line_no,
            column: scanner.column,
        }
    }
}

#[derive(Debug)]
pub struct Scanner {
    pub input: Vec<char>,
    index: usize,
    line_no: usize,
    column: usize,
    start: Option<Pos>,
    past_eof: bool,
}

impl Scanner {
    pub fn from_string(input: &str) -> Self {
        Scanner::new(input.chars().collect())
    }

    pub fn new(input: Vec<char>) -> Self {
        Scanner {
            input,
            index: 0,
            line_no: 0,
            column: 0,
            start: None,
            past_eof: false,
        }
    }

    fn is_eof(&self) -> bool {
        self.index - 1 >= self.input.len()
    }

    fn save_start(&mut self) {
        self.start = Some(Pos::from_scanner(&self))
    }

    fn token_from_start(&mut self, token_type: TokenType) -> Token {
        let pos = self.start.take().unwrap();
        Token {
            token_type,
            start: pos.index,
            line_no: pos.line_no,
            column: pos.column,
            length: self.index - pos.index - 1,
        }
    }

    fn current(&self) -> Option<char> {
        self.input.get(self.index - 1).copied()
    }

    fn look_ahead(&self) -> Option<char> {
        self.input.get(self.index).copied()
    }

    fn next_char(&mut self) -> Option<char> {
        let current = self.input.get(self.index).copied();
        self.index += 1;

        if let Some(current_char) = current {
            if current_char == '\n' {
                self.line_no += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
        }

        current
    }

    // TODO: does this ever return an error? remove `Result` from the return type
    // TODO: if not, also implement `Iterator`
    pub fn next(&mut self) -> Result<Option<Token>> {
        self.skip_whitespace();
        self.save_start();

        if let Some(current) = self.current() {
            match current {
                '"' => {
                    if self.input[self.index - 1..].starts_with(&['"', '"', '"']) {
                        self.next_char();
                        self.next_char();
                        self.long_string().map(Some)
                    } else {
                        self.string().map(Some)
                    }
                }
                '#' => {
                    if self.look_ahead() == Some('#') {
                        self.doc_comment()
                    } else {
                        self.comment()?;
                        self.next()
                    }
                }
                _ => self.word(),
            }
        } else if !self.past_eof {
            self.past_eof = true;
            Ok(Some(self.token_from_start(TokenType::EOF)))
        } else {
            Ok(None)
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(current) = self.next_char() {
            if !current.is_whitespace() {
                break;
            }
        }
    }

    fn doc_comment(&mut self) -> Result<Option<Token>> {
        self.next_char();
        assert_eq!(Some('#'), self.current());
        let mut buffer = Vec::new();

        while let Some(current) = self.next_char() {
            buffer.push(current);
            if current == '\n' || current == '\r' {
                break;
            }
        }

        let token_type = TokenType::DocComment(String::from_iter(&buffer));
        let token = self.token_from_start(token_type);

        Ok(Some(token))
    }

    fn comment(&mut self) -> Result<Option<Token>> {
        while let Some(current) = self.next_char() {
            if current == '\n' || current == '\r' {
                break;
            }
        }
        let token = self.token_from_start(TokenType::Comment);
        Ok(Some(token))
    }

    fn word(&mut self) -> Result<Option<Token>> {
        while let Some(current) = self.next_char() {
            if current.is_whitespace() {
                break;
            }
        }

        let start = self.start.map(|pos| pos.index).unwrap_or_default();
        let end = self.index - 1;
        let word: String = self.input[start..end].iter().collect();
        let token_type = word[..].parse()?;
        let token = self.token_from_start(token_type);

        Ok(Some(token))
    }

    fn long_string(&mut self) -> Result<Token> {
        let word = self.string_until(&['"', '"', '"'])?;
        let token_type = TokenType::String(word);
        let token = self.token_from_start(token_type);
        Ok(token)
    }

    fn string_until(&mut self, terminator: &[char]) -> Result<String> {
        let mut word = String::with_capacity(STRING_INITIALIZATION_CAPACITY);

        while let Some(current) = self.next_char() {
            if self.input[self.index - 1..].starts_with(terminator) {
                for _ in 0..(terminator.len() - 1) {
                    self.next_char();
                }
                break;
            }

            if current == '\\' {
                if let Some(next) = self.next_char() {
                    let char_to_push = match next {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        'u' => {
                            let unichar = self.unicode()?;
                            unichar
                        }
                        c => c,
                    };
                    word.push(char_to_push);
                } else {
                    return Err(Error::EndOfFile(TokenType::String(String::new())));
                }
            } else {
                word.push(current);
            }
        }

        Ok(word)
    }

    fn unicode(&mut self) -> Result<char> {
        if let Some(bracket) = self.next_char() {
            if bracket != '{' {
                return Err(Error::InvalidUnicodeChar);
            }
        } else {
            return Err(Error::InvalidUnicodeChar);
        }

        let hex_str = self.string_until(&['}'])?;

        if !hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::InvalidUnicodeChar);
        }

        let hex_value = u32::from_str_radix(&hex_str, 16).map_err(|_| Error::InvalidUnicodeChar)?;
        let unicode_char = char::from_u32(hex_value).ok_or(Error::InvalidUnicodeChar)?;

        Ok(unicode_char)
    }

    fn string(&mut self) -> Result<Token> {
        let word = self.string_until(&['"'])?;
        let token_type = TokenType::String(word);
        let token = self.token_from_start(token_type);
        Ok(token)
    }

    /// Return all tokens up-to, but not including, the next occurrance of `close`.
    /// If it reaches EOF before this, Returns `Error::EndOfFile(close)`.
    //
    // TODO: make this available to programs for metaprogramming.
    pub fn scan_until(&mut self, close: &TokenType) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while let Some(next) = self.next()? {
            if &next.token_type == close {
                break;
            } else {
                tokens.push(next);
            }
        }

        if self.is_eof() {
            Err(Error::EndOfFile(close.clone()))
        } else {
            Ok(tokens)
        }
    }
}

pub fn scan(input: &str) -> Result<Vec<Token>> {
    let mut scanner = Scanner::from_string(input);
    let mut tokens = Vec::new();

    while let Some(token) = scanner.next()? {
        tokens.push(token)
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests;
