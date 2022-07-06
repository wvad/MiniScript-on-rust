use std::*;
use TokenKind::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifierData {
    pub name: String
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrLiteralData {
    pub value: String
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumLiteralData {
    pub value: f64,
    str_len: usize
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(IdentifierData),

    // Literals
    StrLiteral(StrLiteralData),
    NumLiteral(NumLiteralData),

    // Operators and Symbols
    SingleEqual,
    SemiColon,
    Dot,
    DoubleEqual,
    ExclEqual,
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    LeftParen,
    RightParen,
    LeftCurly,
    RightCurly,
    LeftBracket,
    RightBracket,
    Exclamation
}

impl TokenKind {
    pub fn new_identifier(name: String) -> TokenKind {
        Identifier(IdentifierData { name })
    }
    pub fn new_str_literal(value: String) -> TokenKind {
        StrLiteral(StrLiteralData { value })
    }
    pub fn new_num_literal(value: f64, str_len: usize) -> TokenKind {
        NumLiteral(NumLiteralData { value, str_len })
    }
    pub fn try_into_float(value: &String) -> Result<TokenKind, num::ParseFloatError> {
        <f64 as str::FromStr>::from_str(value).map(|n| Self::new_num_literal(n, value.len()))
    }
    pub fn get_str_len(&self) -> usize {
        match &self {
            StrLiteral(StrLiteralData { value, .. }) => value.len(),
            Identifier(IdentifierData { name, .. }) => name.len(),
            NumLiteral(NumLiteralData { str_len, .. }) => *str_len,
            SingleEqual | SemiColon | LessThan | GreaterThan | Plus | Minus |
            Asterisk | Slash | Percent | LeftParen | RightParen | LeftCurly |
            Dot | RightCurly | LeftBracket | RightBracket | Exclamation => 1,
            DoubleEqual |  ExclEqual |  LessThanEq |  GreaterThanEq => 2
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize
}

#[inline(always)]
fn read_numchars<I: Iterator<Item = char>>(chars: &mut iter::Peekable<I>, out: &mut String) {
    while let Some(c) = chars.next_if(|c| c.is_ascii_digit()) {
        out.push(c);
    }
}

#[inline(always)]
fn parse_number_starting_with_0<I: Iterator<Item = char> + Clone>(chars: &mut iter::Peekable<I>) -> TokenKind {
    match chars.peek() {
        Some('x') => {
            chars.next();
            let mut val: f64 = 0.0;
            let mut len = 2;
            while let Some(c) = chars.next_if(|c| ('0' <= *c && *c <= '9') || ('a' <= *c && *c <= 'f') || ('A' <= *c && *c <= 'F')) {
                val *= 16.0;
                val += u8::from_str_radix(&c.to_string(), 16).unwrap() as f64;
                len += 1;
            }
            TokenKind::new_num_literal(val, len)
        },
        Some('o') => {
            chars.next();
            let mut val: f64 = 0.0;
            let mut len = 2;
            while let Some(c) = chars.next_if(|c| '0' <= *c && *c <= '7') {
                val *= 8.0;
                val += u8::from_str_radix(&c.to_string(), 8).unwrap() as f64;
                len += 1;
            }
            TokenKind::new_num_literal(val, len)
        },
        Some('b') => {
            chars.next();
            let mut val: f64 = 0.0;
            let mut len = 2;
            while let Some(c) = chars.next_if(|c| '0' <= *c && *c <= '1') {
                val *= 2.0;
                val += u8::from_str_radix(&c.to_string(), 2).unwrap() as f64;
                len += 1;
            }
            TokenKind::new_num_literal(val, len)
        },
        Some('.') => {
            let mut text = '0'.to_string();
            if let Some('0'..='9') = chars.clone().nth(1) {
                chars.next();
                text.push('.');
                read_numchars(chars, &mut text);
                if chars.peek() == Some(&'e') {
                    if let Some('1'..='9') = chars.clone().nth(1) {
                        chars.next();
                        text.push('e');
                        read_numchars(chars, &mut text);
                    }
                }
            }
            TokenKind::try_into_float(&text)
            .expect("Invalid float literal")
        },
        _ => TokenKind::new_num_literal(0., 1)
    }
}

struct ParseState {
    tokens: Vec<Token>,
    line: usize,
    column: usize
}

impl ParseState {
    fn new() -> Self {
        Self {
            tokens: Vec::new(),
            line: 1,
            column: 1
        }
    }
    fn push_token(&mut self, kind: TokenKind) {
        let len = kind.get_str_len();
        self.tokens.push(Token {
            kind,
            line: self.line,
            column: self.column
        });
        self.column += len;
    }
}

pub fn parse(input: &str, filename: &str) -> Vec<Token> {
    let mut state = ParseState::new();
    let mut input = input.chars().peekable();
    let show_error = |error: &str, state: &ParseState| -> ! {
        eprintln!("{}\n  at {}:{}:{}", error, filename, state.line, state.column);
        process::exit(1);
    };
    while let Some(c) = input.next() {
        match c {
            '0' => state.push_token(parse_number_starting_with_0(&mut input)),
            '1'..='9' => {
                let mut text = c.to_string();
                read_numchars(&mut input, &mut text);
                if input.peek() == Some(&'.') {
                    if let Some('0'..='9') = input.clone().nth(1) {
                        input.next();
                        text.push('.');
                        read_numchars(&mut input, &mut text);
                    }
                }
                if input.peek() == Some(&'e') {
                    if let Some('1'..='9') = input.clone().nth(1) {
                        input.next();
                        text.push('e');
                        read_numchars(&mut input, &mut text);
                    }
                }
                state.push_token(TokenKind::try_into_float(&text).expect("Invalid float literal"));
            },
            '"' => {
                let mut text = '"'.to_string();
                while let Some(c) = input.next() {
                    match c {
                        '\\' => match input.peek() {
                            Some('n' | 't' | 'r' | '\\' | '"') => {
                                text.push('\\');
                                text.push(input.next().unwrap());
                            },
                            _ => show_error("LiteralError: Invalid escape sequence", &state)
                        },
                        '\n' => show_error("LiteralError: Unterminated string literal", &state),
                        '"' => break,
                        _ => text.push(c)
                    }
                }
                text.push('"');
                state.push_token(TokenKind::new_str_literal(text));
            },
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut text = c.to_string();
                while let Some(c) = input.peek() {
                    match c {
                        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => text.push(input.next().unwrap()),
                        _ => break
                    }
                }
                state.push_token(TokenKind::new_identifier(text));
            },
            '=' => state.push_token(match input.next_if_eq(&'=') {
                None => SingleEqual,
                _ => DoubleEqual
            }),
            '!' => state.push_token(match input.next_if_eq(&'=') {
                None => ExclEqual,
                _ => Exclamation
            }),
            '<' => state.push_token(match input.next_if_eq(&'=') {
                None => LessThan,
                _ => LessThanEq,
            }),
            '>' => state.push_token(match input.next_if_eq(&'=') {
                None => GreaterThan,
                _ => GreaterThanEq
            }),
            '\n' => {
                state.line += 1;
                state.column = 1;
            },
            '+' => state.push_token(Plus),
            '-' => state.push_token(Minus),
            '*' => state.push_token(Asterisk),
            '/' => state.push_token(Slash),
            '%' => state.push_token(Percent),
            ';' => state.push_token(SemiColon),
            '.' => state.push_token(Dot),
            '(' => state.push_token(LeftParen),
            ')' => state.push_token(RightParen),
            '{' => state.push_token(LeftCurly),
            '}' => state.push_token(RightCurly),
            '[' => state.push_token(LeftBracket),
            ']' => state.push_token(RightBracket),
            ' ' | '\t' => state.column += 1,
            '\r' | '\0' => (),
            _ => show_error(&format!("CharacterError: Invalid character '{}'", c), &state)
        }
    }
    state.tokens
}
